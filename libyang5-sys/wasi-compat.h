/*
 * Compatibility shims for building libyang against wasi-libc.
 *
 * Force-included into every translation unit by build.rs, so that the libyang
 * sources themselves need no modification.
 */

#ifdef __wasi__

#include <errno.h>

/*
 * WASI has no dup(). It is only reached through ly_out_new_file() on the
 * LY_OUT_FDSTREAM path, which cannot work without it anyway.
 */
static inline int dup(int fd)
{
    (void)fd;
    errno = ENOSYS;
    return -1;
}

#endif /* __wasi__ */
