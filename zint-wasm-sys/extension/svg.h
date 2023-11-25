#include "../zint/backend/zint.h"

#ifndef SVG_H
#define SVG_H
#ifdef __cplusplus
extern "C" {
#endif /* __cplusplus */
ZINT_EXTERN char *svg_plot_string(struct zint_symbol *symbol, int *error_code);
ZINT_EXTERN void free_svg_plot_string(char *string);
#ifdef __cplusplus
}
#endif /* __cplusplus */

#endif /* SVG_H */
