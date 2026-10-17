/* Zint's filemem.c, with a faster `fm_putsf`.
 *
 * `fm_putsf` writes every coordinate of an SVG, and zint's own formats each one
 * with `sprintf("%.*f")`. wasi-libc formats a float through software emulated
 * long double arithmetic, so under Typst's WebAssembly interpreter printing the
 * SVG costs more than encoding the symbol. The replacement writes the digits
 * from an integer instead, and leaves whatever it cannot do exactly to the
 * original, which is kept under another name.
 *
 * The output is unchanged: the same digits `sprintf` writes, a tie rounded to
 * even, the trailing zeros dropped, and the sign of a negative value that
 * rounds to zero kept, so -0.001 is still written as "-0".
 */

#define fm_putsf zint_wasi_fm_putsf_sprintf
#include "../zint/backend/filemem.c"
#undef fm_putsf

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
