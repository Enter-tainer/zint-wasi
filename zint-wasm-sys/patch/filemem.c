/* Zint's filemem.c, with a faster `fm_putsf` and `fm_printf`.
 *
 * Both write the SVG, and zint's own go through the C library's printf. Under
 * Typst's WebAssembly interpreter a single wasi-libc printf call costs tens of
 * thousands of instructions, and a float through it more, since wasi-libc does
 * the arithmetic in software emulated long double. Printing the SVG cost more
 * than encoding the symbol.
 *
 * `fm_putsf` writes every coordinate. The replacement writes the digits from an
 * integer instead, and leaves whatever it cannot do exactly to the original,
 * which is kept under another name. The output is unchanged: the same digits
 * `sprintf` writes, a tie rounded to even, the trailing zeros dropped, and the
 * sign of a negative value that rounds to zero kept, so -0.001 is still written
 * as "-0".
 *
 * `fm_printf` writes the elements and attributes around them. Every format
 * `svg.c` passes it takes only plain `%d` and `%s`, which the replacement
 * writes itself; any other format goes to the C library as before.
 */

#define fm_putsf zint_wasi_fm_putsf_sprintf
/* Renamed out of the way only; the replacement below falls back to the same
 * `fm_vprintf` and `vfprintf` the original calls. */
#define fm_printf zint_wasi_fm_printf_libc
#include "../zint/backend/filemem.c"
#undef fm_putsf
#undef fm_printf

#include <math.h>
#include <string.h>

/* Room for the longest number `zint_wasi_format_fixed` writes, terminator
 * included. tests/format_fixed.rs relies on it. */
#define ZINT_WASI_FIXED_SIZE 32
/* Up to 9 places, 10^dp needs at most 21 significant bits (5^9 < 2^21), and a
 * float has 24, so scaling it in a double is exact. */
#define ZINT_WASI_FIXED_MAX_DP 9
/* Below 2^53 a double holds every integer, so the rounding below is exact. */
#define ZINT_WASI_FIXED_LIMIT 1e15

/* Writes `arg` with `dp` decimal places, less the trailing zeros, into `buf`.
 * Returns the length written, or 0 for a value it does not cover: an infinity,
 * a NaN, a magnitude of 10^15 or more once scaled, or `dp` outside 0 to 9.
 *
 * Input:  dp 2, 53.340000152587890625f
 * Output: "53.34"
 * Input:  dp 2, 0.125f (a tie)
 * Output: "0.12"
 */
int zint_wasi_format_fixed(char buf[ZINT_WASI_FIXED_SIZE], const int dp, const float arg) {
    char digits[ZINT_WASI_FIXED_SIZE];
    char *p = digits + sizeof(digits);
    unsigned long long scale = 1, scaled, whole, fraction;
    int negative, i, length;
    double magnitude, remainder;

    if (dp < 0 || dp > ZINT_WASI_FIXED_MAX_DP) {
        return 0;
    }
    for (i = 0; i < dp; i++) {
        scale *= 10;
    }

    /* The sign bit rather than `arg < 0`, which is false for -0.0f */
    negative = signbit(arg) != 0;

    magnitude = (double) arg * (double) scale;
    if (negative) {
        magnitude = -magnitude;
    }
    if (!(magnitude < ZINT_WASI_FIXED_LIMIT)) { /* Also false for a NaN */
        return 0;
    }

    scaled = (unsigned long long) magnitude;
    remainder = magnitude - (double) scaled;
    if (remainder > 0.5 || (remainder == 0.5 && (scaled & 1))) {
        scaled++;
    }
    whole = scaled / scale;
    fraction = scaled % scale;

    *--p = '\0';
    if (fraction) {
        int places = dp;
        while (fraction % 10 == 0) {
            fraction /= 10;
            places--;
        }
        while (places-- > 0) {
            *--p = (char) ('0' + fraction % 10);
            fraction /= 10;
        }
        *--p = '.';
    }
    do {
        *--p = (char) ('0' + whole % 10);
        whole /= 10;
    } while (whole);
    if (negative) {
        *--p = '-';
    }

    length = (int) (digits + sizeof(digits) - 1 - p);
    memcpy(buf, p, length + 1);
    return length;
}

INTERNAL int fm_putsf(const char *prefix, const int dp, const float arg, struct filemem *restrict const fmp) {
    char buf[ZINT_WASI_FIXED_SIZE];

    if (!zint_wasi_format_fixed(buf, dp, arg)) {
        return zint_wasi_fm_putsf_sprintf(prefix, dp, arg, fmp);
    }

    assert(fmp);
    if (fmp->err) {
        return 0;
    }
    if (prefix && *prefix) {
        if (!fm_puts(prefix, fmp)) {
            return 0;
        }
    }
    return fm_puts(buf, fmp);
}

/* Whether `fmt` holds no conversion but a bare `%d` or `%s`.
 *
 * Input:  " <g id=\"barcode\" fill=\"#%s\">\n"  -> 1
 * Input:  "%02X%02X%02X"                       -> 0
 */
static int zint_wasi_is_plain_format(const char *fmt) {
    for (; *fmt; fmt++) {
        if (*fmt == '%') {
            fmt++;
            if (*fmt != 'd' && *fmt != 's') {
                return 0;
            }
        }
    }
    return 1;
}

/* Writes a format `zint_wasi_is_plain_format` accepts. */
static int zint_wasi_fm_vprintf_plain(struct filemem *restrict const fmp, const char *fmt, va_list ap) {
    char digits[12]; /* "-2147483648" and the terminator */

    while (*fmt) {
        const char *literal = fmt;
        while (*fmt && *fmt != '%') {
            fmt++;
        }
        if (fmt != literal && !fm_write(literal, 1, (size_t) (fmt - literal), fmp)) {
            return 0;
        }
        if (!*fmt) {
            break;
        }

        if (fmt[1] == 's') {
            if (!fm_puts(va_arg(ap, const char *), fmp)) {
                return 0;
            }
        } else {
            const int value = va_arg(ap, int);
            /* Through unsigned, so INT_MIN negates without overflowing */
            unsigned int magnitude = value < 0 ? 0u - (unsigned int) value : (unsigned int) value;
            char *p = digits + sizeof(digits);

            *--p = '\0';
            do {
                *--p = (char) ('0' + magnitude % 10);
                magnitude /= 10;
            } while (magnitude);
            if (value < 0) {
                *--p = '-';
            }
            if (!fm_puts(p, fmp)) {
                return 0;
            }
        }
        fmt += 2;
    }
    return 1;
}

/* What zint's own `fm_printf` does with its arguments. */
static int zint_wasi_fm_vprintf_libc(struct filemem *restrict const fmp, const char *fmt, va_list ap) {
    if (fmp->flags & BARCODE_MEMORY_FILE) {
        return fm_vprintf(fmp, fmt, ap);
    }
    return vfprintf(fmp->fp, fmt, ap) >= 0 ? 1 : fm_seterr(fmp, errno);
}

static int zint_wasi_fm_vprintf(struct filemem *restrict const fmp, const char *fmt, va_list ap) {
    assert(fmp && fmt);
    if (fmp->err) {
        return 0;
    }
    if (zint_wasi_is_plain_format(fmt)) {
        return zint_wasi_fm_vprintf_plain(fmp, fmt, ap);
    }
    return zint_wasi_fm_vprintf_libc(fmp, fmt, ap);
}

INTERNAL int fm_printf(struct filemem *restrict const fmp, const char *fmt, ...) {
    va_list ap;
    int ret;

    va_start(ap, fmt);
    ret = zint_wasi_fm_vprintf(fmp, fmt, ap);
    va_end(ap);
    return ret;
}

/* For tests/fm_printf.rs: writes one `fm_printf` into the in-memory file of
 * `symbol`, through the replacement when `fast` is set and through zint's own
 * `fm_vprintf` otherwise, so the two can be compared. */
int zint_wasi_fm_printf_to_memory(struct zint_symbol *symbol, const int fast, const char *fmt, ...) {
    struct filemem fm;
    va_list ap;
    int ret;

    if (!fm_open(&fm, symbol, "w")) {
        return 0;
    }
    va_start(ap, fmt);
    ret = fast ? zint_wasi_fm_vprintf(&fm, fmt, ap) : fm_vprintf(&fm, fmt, ap);
    va_end(ap);
    return fm_close(&fm, symbol) && ret;
}
