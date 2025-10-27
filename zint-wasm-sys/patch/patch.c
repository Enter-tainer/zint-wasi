#include "../zint/backend/zint.h"

#ifdef DISABLE_RASTER
int zint_plot_raster(struct zint_symbol *symbol, int rotate_angle, int file_type) {
    return 0;
}
#endif
#ifdef DISABLE_VECTOR
int zint_plot_vector(struct zint_symbol *symbol, int rotate_angle, int file_type) {
    return 0;
}
#endif
#ifdef ZINT_NO_PS
int zint_ps_plot(struct zint_symbol *symbol, int rotate_angle) {
    return 0;
}
#endif
#ifdef ZINT_NO_EMF
int zint_emf_plot(struct zint_symbol *symbol, int rotate_angle) {
    return 0;
}
#endif
#ifdef ZINT_NO_SVG
int zint_svg_plot(struct zint_symbol *symbol, int rotate_angle) { return 0; }
#endif
