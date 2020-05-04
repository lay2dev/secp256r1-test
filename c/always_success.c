/* for testing purpose */

#include "libsig.h"

int main()
{
    int ret;
    ret = 0;

    char *ec_name = "SECP256R1";
    u8 curve_name_len = 10;

    const ec_str_params *curve_params =
        ec_get_curve_params_by_name((const u8 *)ec_name, curve_name_len);

    if (curve_params == NULL)
    {
        return 11;
    }

    ec_params params;
    import_params(&params, curve_params);

    u8 pub_key_buffer[] = {
        0, 1, 4, 212, 48, 198, 234, 164, 253, 229, 14, 223, 237, 6, 18, 78, 207, 197, 224, 206, 142, 9, 77, 128, 44, 100, 84, 239, 245, 147, 128, 15, 152, 171, 229, 54, 163, 146, 203, 152, 213, 198, 2, 39, 12, 84, 227, 213, 69, 224, 118, 131, 46, 160, 62, 86, 92, 60, 208, 46, 3, 177, 223, 164, 137, 114, 44, 149, 111, 18, 114, 130, 162, 134, 121, 174, 212, 69, 151, 58, 26, 104, 224, 21, 141, 220, 215, 26, 200, 160, 162, 113, 201, 85, 142, 192, 33, 109, 208};
    ec_pub_key pub_key;
    ec_structured_pub_key_import_from_buf(&pub_key, &params, pub_key_buffer, 99, 1);

    const u8 sig[] = {
        0xCB, 0x28, 0xE0, 0x99, 0x9B, 0x9C, 0x77, 0x15,
        0xFD, 0x0A, 0x80, 0xD8, 0xE4, 0x7A, 0x77, 0x07,
        0x97, 0x16, 0xCB, 0xBF, 0x91, 0x7D, 0xD7, 0x2E,
        0x97, 0x56, 0x6E, 0xA1, 0xC0, 0x66, 0x95, 0x7C,
        0x86, 0xFA, 0x3B, 0xB4, 0xE2, 0x6C, 0xAD, 0x5B,
        0xF9, 0x0B, 0x7F, 0x81, 0x89, 0x92, 0x56, 0xCE,
        0x75, 0x94, 0xBB, 0x1E, 0xA0, 0xC8, 0x92, 0x12,
        0x74, 0x8B, 0xFF, 0x3B, 0x3D, 0x5B, 0x03, 0x15};
    u8 siglen = 64;
    char *m = "abc";


    struct ec_verify_context ctx;
    ec_verify_init(&ctx, &pub_key, sig, siglen, 1, 2);
    ec_verify_update(&ctx, (const u8 *)m, 3);

    ret = ec_verify_finalize(&ctx);

    return ret;
}
