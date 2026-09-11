#include "../zint/backend/zint.h"
// we only need svg plot, so patch the rest to empty functions. Each keeps the
// signature zint declares for it.
int plot_raster(struct zint_symbol *symbol, int rotate_angle, int file_type) {
    return 0;
}
int ps_plot(struct zint_symbol *symbol) {
    return 0;
}
int emf_plot(struct zint_symbol *symbol, int rotate_angle) {
    return 0;
}
