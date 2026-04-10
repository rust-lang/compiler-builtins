/* Correctly-rounded inverse hyperbolic cosine function for the
   binary64 floating point format.

Copyright (c) 2023 Alexei Sibidanov.

This file is part of the CORE-MATH project
(https://core-math.gitlabpages.inria.fr/).

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE. */

/* References:
   [1] Tight and rigourous error bounds for basic building blocks of
       double-word arithmetic, by Mioara Joldeş, Jean-Michel Muller,
       and Valentina Popescu, ACM Transactions on Mathematical Software,
       44(2), 2017.
*/

use super::support::{CastInto, Float, Int, cold_path, likely};

pub fn acosh(x: f64) -> f64 {
    cr_acoshf64(x)
}

fn fasttwosum(x: f64, y: f64) -> (f64, f64) {
    let s = x + y;
    let z = s - x;
    let e = y - z;
    (s, e)
}

fn adddd(xh: f64, xl: f64, ch: f64, cl: f64) -> (f64, f64) {
    let s = xh + ch;
    let d = s - xh;
    let l = ((ch - d) + (xh + (d - s))) + (xl + cl);
    (s, l)
}

/* This function implements Algorithm 10 (DWTimesDW1) from [1]
Its relative error (for round-to-nearest ties-to-even) is bounded by 7u^2
(Theorem 5.1 of [1]), where u = 2^-53 for double precision,
assuming xh = RN(xh + xl), which implies |xl| <= 1/2 ulp(xh),
and similarly for ch, cl. */
fn muldd_acc(xh: f64, xl: f64, ch: f64, cl: f64) -> (f64, f64) {
    let ahlh = ch * xl;
    let alhh = cl * xh;
    let ahhh = ch * xh;
    let mut ahhl = ch.fma(xh, -ahhh);
    ahhl += alhh + ahlh;
    fasttwosum(ahhh, ahhl)
}

fn mulddd(xh: f64, xl: f64, mut ch: f64) -> (f64, f64) {
    let ahlh = ch * xl;
    let ahhh = ch * xh;
    let mut ahhl = ch.fma(xh, -ahhh);
    ahhl += ahlh;
    ch = ahhh + ahhl;
    let l = (ahhh - ch) + ahhl;
    (ch, l)
}

fn polydd(xh: f64, xl: f64, n: i32, c: &[(f64, f64)], l: f64) -> (f64, f64) {
    let mut i = n - 1;
    let mut ch = c[i as usize].0 + l;
    let mut cl = ((c[i as usize].0 - ch) + l) + c[i as usize].1;
    while i > 0 {
        i -= 1;
        (ch, cl) = muldd_acc(xh, xl, ch, cl);
        let th = ch + c[i as usize].0;
        let tl = (c[i as usize].0 - th) + ch;
        ch = th;
        cl += tl + c[i as usize].1;
    }
    (ch, cl)
}

#[inline(never)]
fn as_acosh_one(x: f64, sh: f64, sl: f64) -> f64 {
    #[rustfmt::skip]
    static CH: &[(f64, f64)] = &[
        (hf64!("-0x1.5555555555555p-4"), hf64!("-0x1.5555555554af1p-58")),
        (hf64!("0x1.3333333333333p-6"), hf64!("0x1.9999998933f0ep-61")),
        (hf64!("-0x1.6db6db6db6db7p-8"), hf64!("0x1.24929b16ec6b7p-63")),
        (hf64!("0x1.f1c71c71c71c7p-10"), hf64!("0x1.c56d45e265e2cp-66")),
        (hf64!("-0x1.6e8ba2e8ba2e9p-11"), hf64!("0x1.6d50ce7188d3dp-65")),
        (hf64!("0x1.1c4ec4ec4ec43p-12"), hf64!("0x1.c6791d1cf399ap-66")),
        (hf64!("-0x1.c99999999914fp-14"), hf64!("0x1.ee0d9408a2e2ap-68")),
        (hf64!("0x1.7a878787648e2p-15"), hf64!("-0x1.1cea281e08012p-69")),
        (hf64!("-0x1.3fde50d0cb4b9p-16"), hf64!("0x1.0335101403d9dp-72")),
        (hf64!("0x1.12ef3bf8a0a74p-17"), hf64!("0x1.f9c6b51787043p-80")),
    ];

    let cl = [
        hf64!("-0x1.df3b9d1296ea9p-19"),
        hf64!("0x1.a681d7d2298ebp-20"),
        hf64!("-0x1.77ead7b1ca449p-21"),
        hf64!("0x1.4edd2ddb3721fp-22"),
        hf64!("-0x1.1bf173531ee23p-23"),
        hf64!("0x1.613229230e255p-25"),
    ];

    let mut y0;
    let mut y1;
    let mut y2 = x * (cl[0] + x * (cl[1] + x * (cl[2] + x * (cl[3] + x * (cl[4] + x * (cl[5]))))));
    (y1, y2) = polydd(x, 0.0, 10, CH, y2);
    (y1, y2) = mulddd(y1, y2, x);
    (y0, y1) = fasttwosum(1.0, y1);
    y1 += y2;
    (y0, y1) = muldd_acc(y0, y1, sh, sl);
    y0 + y1
}

static B: &[(u16, i16)] = &[
    (301, 27565),
    (7189, 24786),
    (13383, 22167),
    (18923, 19696),
    (23845, 17361),
    (28184, 15150),
    (31969, 13054),
    (35231, 11064),
    (37996, 9173),
    (40288, 7372),
    (42129, 5657),
    (43542, 4020),
    (44546, 2457),
    (45160, 962),
    (45399, -468),
    (45281, -1838),
    (44821, -3151),
    (44032, -4412),
    (42929, -5622),
    (41522, -6786),
    (39825, -7905),
    (37848, -8982),
    (35602, -10020),
    (33097, -11020),
    (30341, -11985),
    (27345, -12916),
    (24115, -13816),
    (20661, -14685),
    (16989, -15526),
    (13107, -16339),
    (9022, -17126),
    (4740, -17889),
];
static R1: &[f64] = &[
    hf64!("0x1p+0"),
    hf64!("0x1.f5076p-1"),
    hf64!("0x1.ea4bp-1"),
    hf64!("0x1.dfc98p-1"),
    hf64!("0x1.d5818p-1"),
    hf64!("0x1.cb72p-1"),
    hf64!("0x1.c199cp-1"),
    hf64!("0x1.b7f76p-1"),
    hf64!("0x1.ae8ap-1"),
    hf64!("0x1.a5504p-1"),
    hf64!("0x1.9c492p-1"),
    hf64!("0x1.93738p-1"),
    hf64!("0x1.8ace6p-1"),
    hf64!("0x1.8258ap-1"),
    hf64!("0x1.7a114p-1"),
    hf64!("0x1.71f76p-1"),
    hf64!("0x1.6a09ep-1"),
    hf64!("0x1.6247ep-1"),
    hf64!("0x1.5ab08p-1"),
    hf64!("0x1.5342cp-1"),
    hf64!("0x1.4bfdap-1"),
    hf64!("0x1.44e08p-1"),
    hf64!("0x1.3dea6p-1"),
    hf64!("0x1.371a8p-1"),
    hf64!("0x1.306fep-1"),
    hf64!("0x1.29e9ep-1"),
    hf64!("0x1.2387ap-1"),
    hf64!("0x1.1d488p-1"),
    hf64!("0x1.172b8p-1"),
    hf64!("0x1.11302p-1"),
    hf64!("0x1.0b558p-1"),
    hf64!("0x1.059bp-1"),
    hf64!("0x1p-1"),
];
static R2: &[f64] = &[
    hf64!("0x1p+0"),
    hf64!("0x1.ffa74p-1"),
    hf64!("0x1.ff4eap-1"),
    hf64!("0x1.fef62p-1"),
    hf64!("0x1.fe9dap-1"),
    hf64!("0x1.fe452p-1"),
    hf64!("0x1.fdeccp-1"),
    hf64!("0x1.fd946p-1"),
    hf64!("0x1.fd3c2p-1"),
    hf64!("0x1.fce3ep-1"),
    hf64!("0x1.fc8bcp-1"),
    hf64!("0x1.fc33ap-1"),
    hf64!("0x1.fbdbap-1"),
    hf64!("0x1.fb83ap-1"),
    hf64!("0x1.fb2bcp-1"),
    hf64!("0x1.fad3ep-1"),
    hf64!("0x1.fa7c2p-1"),
    hf64!("0x1.fa246p-1"),
    hf64!("0x1.f9ccap-1"),
    hf64!("0x1.f975p-1"),
    hf64!("0x1.f91d8p-1"),
    hf64!("0x1.f8c6p-1"),
    hf64!("0x1.f86e8p-1"),
    hf64!("0x1.f8172p-1"),
    hf64!("0x1.f7bfep-1"),
    hf64!("0x1.f768ap-1"),
    hf64!("0x1.f7116p-1"),
    hf64!("0x1.f6ba4p-1"),
    hf64!("0x1.f6632p-1"),
    hf64!("0x1.f60c2p-1"),
    hf64!("0x1.f5b52p-1"),
    hf64!("0x1.f55e4p-1"),
    hf64!("0x1.f5076p-1"),
];
static L1: &[(f64, f64)] = &[
    (hf64!("0x0p+0"), hf64!("0x0p+0")),
    (hf64!("-0x1.269e2038315b3p-46"), hf64!("0x1.62e4eacd4p-6")),
    (hf64!("-0x1.3f2558bddfc47p-45"), hf64!("0x1.62e3ce7218p-5")),
    (hf64!("0x1.07ea13c34efb5p-45"), hf64!("0x1.0a2ab6d3ecp-4")),
    (hf64!("0x1.8f3e77084d3bap-44"), hf64!("0x1.62e4a86d8cp-4")),
    (hf64!("-0x1.8d92a005f1a7ep-46"), hf64!("0x1.bb9db7062cp-4")),
    (hf64!("0x1.58239e799bfe5p-44"), hf64!("0x1.0a2b1a22ccp-3")),
    (hf64!("-0x1.a93fcf5f593b7p-44"), hf64!("0x1.3687f0a298p-3")),
    (hf64!("-0x1.db4cac32fd2b5p-46"), hf64!("0x1.62e4116b64p-3")),
    (hf64!("-0x1.0e65a92ee0f3bp-46"), hf64!("0x1.8f409e4df6p-3")),
    (hf64!("-0x1.8261383d475f1p-44"), hf64!("0x1.bb9d15001cp-3")),
    (hf64!("-0x1.359886207513bp-44"), hf64!("0x1.e7f9a8c94p-3")),
    (hf64!("0x1.811f87496ceb7p-44"), hf64!("0x1.0a2b052ddbp-2")),
    (hf64!("0x1.4991ec6cb435cp-44"), hf64!("0x1.205955ef73p-2")),
    (hf64!("-0x1.4581abfeb8927p-44"), hf64!("0x1.3687bd9121p-2")),
    (hf64!("0x1.cab48f6942703p-44"), hf64!("0x1.4cb5e8f2b5p-2")),
    (hf64!("-0x1.df2c452fde132p-47"), hf64!("0x1.62e4420e2p-2")),
    (hf64!("0x1.6109f4fdb74bdp-45"), hf64!("0x1.791292c46ap-2")),
    (hf64!("-0x1.6b95fbdac7696p-44"), hf64!("0x1.8f40af84e7p-2")),
    (hf64!("0x1.7394fa880cbdap-46"), hf64!("0x1.a56ed8f865p-2")),
    (hf64!("-0x1.50b06a94eccabp-46"), hf64!("0x1.bb9d6505b4p-2")),
    (hf64!("-0x1.be2abf0b38989p-44"), hf64!("0x1.d1cb91e728p-2")),
    (hf64!("-0x1.7d6bf1e34da04p-44"), hf64!("0x1.e7f9d139e2p-2")),
    (hf64!("-0x1.423c1e14de6edp-44"), hf64!("0x1.fe27db9b0ep-2")),
    (hf64!("0x1.c46f1a0efbbc2p-44"), hf64!("0x1.0a2b25060a8p-1")),
    (hf64!("0x1.834fe4e3e6018p-45"), hf64!("0x1.154244482ap-1")),
    (hf64!("0x1.6a03d0f02b65p-46"), hf64!("0x1.20597312988p-1")),
    (hf64!("0x1.d437056526f3p-44"), hf64!("0x1.2b707145dep-1")),
    (hf64!("-0x1.a0233728405c5p-45"), hf64!("0x1.3687b0e0b28p-1")),
    (hf64!("-0x1.4dbdda10d2bf1p-45"), hf64!("0x1.419ec5d3f68p-1")),
    (hf64!("0x1.f7d0a25d154f2p-44"), hf64!("0x1.4cb5f9fc02p-1")),
    (hf64!("0x1.15ede4d803b18p-44"), hf64!("0x1.57cd28421a8p-1")),
    (hf64!("0x1.ef35793c7673p-45"), hf64!("0x1.62e42fefa38p-1")),
];
static L2: &[(f64, f64)] = &[
    (hf64!("0x0p+0"), hf64!("0x0p+0")),
    (hf64!("0x1.5abdac3638e99p-44"), hf64!("0x1.631ec81ep-11")),
    (hf64!("-0x1.16b8be9bbe239p-45"), hf64!("0x1.62fd8127p-10")),
    (hf64!("-0x1.364c6315542ebp-44"), hf64!("0x1.0a2520508p-9")),
    (hf64!("0x1.734abe459c9p-45"), hf64!("0x1.62dadc1dp-9")),
    (hf64!("0x1.0cf8a761431bfp-44"), hf64!("0x1.bb9ff94dp-9")),
    (hf64!("0x1.da2718eb78708p-45"), hf64!("0x1.0a2a2def8p-8")),
    (hf64!("0x1.34ada62c59b93p-44"), hf64!("0x1.368c0fae4p-8")),
    (hf64!("0x1.d09ab376682d4p-44"), hf64!("0x1.62e58e4f8p-8")),
    (hf64!("-0x1.3cb7b94329211p-45"), hf64!("0x1.8f46bd28cp-8")),
    (hf64!("-0x1.eec5c297c41dp-45"), hf64!("0x1.bb9f8312p-8")),
    (hf64!("-0x1.6411b9395d15p-44"), hf64!("0x1.e7fff8f3p-8")),
    (hf64!("-0x1.1c0e59a43053cp-44"), hf64!("0x1.0a2c0006ep-7")),
    (hf64!("0x1.6506596e077b6p-46"), hf64!("0x1.205bdb6fp-7")),
    (hf64!("0x1.e256bce6faa27p-44"), hf64!("0x1.36877c86ep-7")),
    (hf64!("0x1.bd42467b0c8d1p-51"), hf64!("0x1.4cb6f5578p-7")),
    (hf64!("-0x1.c4f92132ff0fp-44"), hf64!("0x1.62e230e8cp-7")),
    (hf64!("-0x1.80be08bfab39p-44"), hf64!("0x1.7911440f6p-7")),
    (hf64!("-0x1.f0b1319ceb1f7p-44"), hf64!("0x1.8f443020ap-7")),
    (hf64!("0x1.a65fcfb8de99bp-45"), hf64!("0x1.a572dbef4p-7")),
    (hf64!("0x1.4233885d3779cp-46"), hf64!("0x1.bb9d449a6p-7")),
    (hf64!("0x1.f46a59e646edbp-44"), hf64!("0x1.d1cb8491cp-7")),
    (hf64!("-0x1.c3d2f11c11446p-44"), hf64!("0x1.e7fd9d2aap-7")),
    (hf64!("0x1.7763f78a1e0ccp-45"), hf64!("0x1.fe2b6f978p-7")),
    (hf64!("0x1.b4c37fc60c043p-44"), hf64!("0x1.0a2a7c7a5p-6")),
    (hf64!("-0x1.5b8a822859be3p-46"), hf64!("0x1.15412ca86p-6")),
    (hf64!("-0x1.f2d8c9fc064p-44"), hf64!("0x1.2059c9005p-6")),
    (hf64!("-0x1.e80e79c20378dp-44"), hf64!("0x1.2b703f49bp-6")),
    (hf64!("0x1.68256e4329bdbp-44"), hf64!("0x1.3688a1a8dp-6")),
    (hf64!("0x1.7e9741da248c3p-44"), hf64!("0x1.419edc7bap-6")),
    (hf64!("0x1.e330dccce602bp-45"), hf64!("0x1.4cb7034fap-6")),
    (hf64!("0x1.2f32b5d18eefbp-49"), hf64!("0x1.57cd01187p-6")),
    (hf64!("-0x1.269e2038315b3p-46"), hf64!("0x1.62e4eacd4p-6")),
];
static C: &[f64] = &[
    hf64!("-0x1p-1"),
    hf64!("0x1.555555555553p-2"),
    hf64!("-0x1.fffffffffffap-3"),
    hf64!("0x1.99999e33a6366p-3"),
    hf64!("-0x1.555559ef9525fp-3"),
];

fn cr_acoshf64(x: f64) -> f64 {
    let ix = x.to_bits();
    if ix >= 0x7ff0000000000000_u64 {
        // x<0 or NaN/Inf
        cold_path();
        let aix: u64 = ix << 1;
        if ix == 0x7ff0000000000000_u64 || aix > (0x7ff_u64 << 53) {
            return x + x; // +inf or nan
        } // #ifdef CORE_MATH_SUPPORT_ERRNO
        //       errno = EDOM;
        // #endif
        return f64::nan("x<1");
    }

    if ix.signed() <= 0x3ff0000000000000_i64 {
        cold_path();
        if ix == 0x3ff0000000000000_u64 {
            return 0.0;
        }
        // #ifdef CORE_MATH_SUPPORT_ERRNO
        //     errno = EDOM;
        // #endif
        return f64::nan("x<1");
    }
    let g: f64;
    let mut off: i32 = 0x3fe;
    let mut t = ix;
    if ix < 0x3ff1e83e425aee63_u64 {
        // 0 <= x < hf64!("0x1.1e83e425aee63p+0")
        let z = x - 1.0;
        let iz = (-0.25) / z;
        let zt = 2.0 * z;
        let sh = zt.sqrt();
        let sl = sh.fma(sh, -zt) * (sh * iz);
        let cl = [
            hf64!("-0x1.5555555555555p-4"),
            hf64!("0x1.3333333332f95p-6"),
            hf64!("-0x1.6db6db6d5534cp-8"),
            hf64!("0x1.f1c71c1e04356p-10"),
            hf64!("-0x1.6e8b8e3e40d58p-11"),
            hf64!("0x1.1c4ba825ac4fep-12"),
            hf64!("-0x1.c9045534e6d9ep-14"),
            hf64!("0x1.71fedae26a76bp-15"),
            hf64!("-0x1.f1f4f8cc65342p-17"),
        ];
        let z2 = z * z;
        let z4 = z2 * z2;
        let ds = (sh * z).fma(
            cl[0]
                + z * (((cl[1] + z * cl[2]) + z2 * (cl[3] + z * cl[4]))
                    + z4 * ((cl[5] + z * cl[6]) + z2 * (cl[7] + z * cl[8]))),
            sl,
        );
        let eps = ds * hf64!("0x1.fcp-51") - hf64!("0x1p-104") * sh;
        let lb = sh + (ds - eps);
        let ub = sh + (ds + eps);
        if lb == ub {
            return lb;
        }
        return as_acosh_one(z, sh, sl);
    } else if likely(ix < 0x405bf00000000000_u64) {
        /* hf64!("0x1.1e83e425aee63p+0") <= x < hf64!("0x1.bfp+6"): this branch was checked
        exhaustively (revision 1bd85b8) with/without FMA */
        off = 0x3ff;
        let x2h = x * x;
        let wh = x2h - 1.0;
        let wl = x.fma(x, -x2h);
        let sh = wh.sqrt();
        let ish = 0.5 / wh;
        let sl = (wl - sh.fma(sh, -wh)) * (sh * ish);
        let (th, mut tl) = fasttwosum(x, sh);
        tl += sl;
        t = th.to_bits();
        g = tl / th;
    } else if ix < 0x4087100000000000_u64 {
        // hf64!("0x1.bfp+6") <= x < hf64!("0x1.71p+9")
        /* this branch was tested exhaustively (revision 28faf30) with/without FMA */
        let cl = [
            hf64!("0x1.5c4b6148816e2p-66"),
            hf64!("-0x1.000000000005cp-2"),
            hf64!("-0x1.7fffffebf3e6cp-4"),
            hf64!("-0x1.aab6691f2bae7p-5"),
        ];
        let z = 1.0 / (x * x);
        g = cl[0] + z * (cl[1] + z * (cl[2] + z * cl[3]));
    } else if ix < 0x40e0100000000000_u64 {
        // hf64!("0x1.71p+9") <= x < hf64!("0x1.01p+15")
        /* this branch was tested exhaustively (revision d764c73) with/without FMA */
        let cl = [
            hf64!("-0x1.7f77c8429c6c6p-67"),
            hf64!("-0x1.ffffffffff214p-3"),
            hf64!("-0x1.8000268641bfep-4"),
        ];
        let z = 1.0 / (x * x);
        g = cl[0] + z * (cl[1] + z * cl[2]);
    } else if ix < 0x41ea000000000000_u64 {
        // hf64!("0x1.01p+15") <= x < hf64!("0x1.ap+31")
        /* tested exhaustively (revision d764c73) with/without FMA:
           hf64!("0x1.01p+15") <= x < 2^16
        */
        let cl = [
            hf64!("0x1.7a0ed2effdd1p-67"),
            hf64!("-0x1.000000017d048p-2"),
        ];
        let z = 1.0 / (x * x);
        g = cl[0] + z * cl[1];
    } else {
        // hf64!("0x1.ap+31") <= x
        g = 0.0;
    }
    let ex: i32 = (t >> 52).cast();
    let e: i32 = ex - off;
    t &= u64::MAX >> 12;
    let ed: f64 = e.cast();
    let i = t >> (52 - 5);
    let d: i64 = (t & (u64::MAX >> 17)).signed();
    let j: u64 = (t
        .wrapping_add(u64::from(B[i as usize].0) << 33)
        .wrapping_add((i64::from(B[i as usize].1) * (d >> 16)).unsigned()))
        >> (52 - 10);
    t |= 0x3ff_u64 << 52;
    let i1: i32 = (j >> 5).cast();
    let i2: i32 = (j & 0x1f).cast();
    let r: f64 = R1[i1 as usize] * R2[i2 as usize];
    let dx = r.fma(f64::from_bits(t), -1.0);
    let dx2 = dx * dx;
    let f: f64 = dx2 * ((C[0] + dx * C[1]) + dx2 * ((C[2] + dx * C[3]) + dx2 * C[4]));
    let l2h = hf64!("0x1.62e42fefa38p-1");
    let l2l = hf64!("0x1.ef35793c7673p-45");
    let lh = (L1[i1 as usize].1 + L2[i2 as usize].1) + l2h * ed;
    let mut ll = dx + l2l * ed;
    ll += g;
    ll += L1[i1 as usize].0 + L2[i2 as usize].0;
    ll += f;
    let eps = 2.8e-19;
    let lb = lh + (ll - eps);
    let ub = lh + (ll + eps);
    if lb == ub {
        return lb;
    } else {
        cold_path();
        as_acosh_refine(x, hf64!("0x1.71547652b82fep+0") * lb)
    }
}

#[inline(never)]
fn as_acosh_database(x: f64, mut f: f64) -> f64 {
    #[rustfmt::skip]
    static DB: &[(f64, f64, f64)] = &[
        (hf64!("0x1.5bff041b260fep+0"), hf64!("0x1.a6031cd5f93bap-1"), hf64!("0x1p-55")),
        (hf64!("0x1.9efdca62b700ap+0"), hf64!("0x1.104b648f113a1p+0"), hf64!("0x1p-54")),
        (hf64!("0x1.9efdca62b700ap+0"), hf64!("0x1.104b648f113a1p+0"), hf64!("0x1p-54")),
        (hf64!("0x1.a5bf3acfde4b2p+0"), hf64!("0x1.1585720f35cd9p+0"), hf64!("-0x1p-54")),
        (hf64!("0x1.d888dd2101d93p+1"), hf64!("0x1.faf8b7a12cf9fp+0"), hf64!("-0x1p-54")),
        (hf64!("0x1.0151def34c2b8p+5"), hf64!("0x1.0a7b6e3fed72p+2"), hf64!("0x1p-52")),
        (hf64!("0x1.45ea160ddc71fp+7"), hf64!("0x1.725811dcf6782p+2"), hf64!("0x1p-52")),
        (hf64!("0x1.13570067acc9fp+9"), hf64!("0x1.c04672343dccfp+2"), hf64!("-0x1p-52")),
        (hf64!("0x1.2a686e4b567cep+10"), hf64!("0x1.f1c928e7f1e65p+2"), hf64!("0x1p-52")),
        (hf64!("0x1.cb62eec26bd78p+15"), hf64!("0x1.759a2ad4c4d56p+3"), hf64!("0x1p-51")),
    ];
    let mut a: i32 = 0;
    let mut b: i32 = DB.len() as i32 - 1;
    let mut m = (a + b) / 2;
    while a <= b {
        // binary search
        if DB[m as usize].0 < x {
            a = m + 1;
        } else if DB[m as usize].0 == x {
            f = DB[m as usize].1 + DB[m as usize].2;
            break;
        } else {
            b = m - 1;
        }
        m = (a + b) / 2;
    }
    f
}

static T1: &[f64] = &[
    hf64!("0x1p+0"),
    hf64!("0x1.ea4afap-1"),
    hf64!("0x1.d5818ep-1"),
    hf64!("0x1.c199bep-1"),
    hf64!("0x1.ae89f98p-1"),
    hf64!("0x1.9c4918p-1"),
    hf64!("0x1.8ace54p-1"),
    hf64!("0x1.7a1147p-1"),
    hf64!("0x1.6a09e68p-1"),
    hf64!("0x1.5ab07ep-1"),
    hf64!("0x1.4bfdad8p-1"),
    hf64!("0x1.3dea65p-1"),
    hf64!("0x1.306fe08p-1"),
    hf64!("0x1.2387a7p-1"),
    hf64!("0x1.172b84p-1"),
    hf64!("0x1.0b5587p-1"),
    hf64!("0x1p-1"),
];
static T2: &[f64] = &[
    hf64!("0x1p+0"),
    hf64!("0x1.fe9d968p-1"),
    hf64!("0x1.fd3c228p-1"),
    hf64!("0x1.fbdba38p-1"),
    hf64!("0x1.fa7c18p-1"),
    hf64!("0x1.f91d8p-1"),
    hf64!("0x1.f7bfdbp-1"),
    hf64!("0x1.f663278p-1"),
    hf64!("0x1.f507658p-1"),
    hf64!("0x1.f3ac948p-1"),
    hf64!("0x1.f252b38p-1"),
    hf64!("0x1.f0f9c2p-1"),
    hf64!("0x1.efa1bfp-1"),
    hf64!("0x1.ee4aaap-1"),
    hf64!("0x1.ecf483p-1"),
    hf64!("0x1.eb9f488p-1"),
];
static T3: &[f64] = &[
    hf64!("0x1p+0"),
    hf64!("0x1.ffe9d2p-1"),
    hf64!("0x1.ffd3a58p-1"),
    hf64!("0x1.ffbd798p-1"),
    hf64!("0x1.ffa74e8p-1"),
    hf64!("0x1.ff91248p-1"),
    hf64!("0x1.ff7afb8p-1"),
    hf64!("0x1.ff64d38p-1"),
    hf64!("0x1.ff4eac8p-1"),
    hf64!("0x1.ff38868p-1"),
    hf64!("0x1.ff22618p-1"),
    hf64!("0x1.ff0c3dp-1"),
    hf64!("0x1.fef61ap-1"),
    hf64!("0x1.fedff78p-1"),
    hf64!("0x1.fec9d68p-1"),
    hf64!("0x1.feb3b6p-1"),
];
static T4: &[f64] = &[
    hf64!("0x1p+0"),
    hf64!("0x1.fffe9dp-1"),
    hf64!("0x1.fffd3ap-1"),
    hf64!("0x1.fffbd78p-1"),
    hf64!("0x1.fffa748p-1"),
    hf64!("0x1.fff9118p-1"),
    hf64!("0x1.fff7ae8p-1"),
    hf64!("0x1.fff64cp-1"),
    hf64!("0x1.fff4e9p-1"),
    hf64!("0x1.fff386p-1"),
    hf64!("0x1.fff2238p-1"),
    hf64!("0x1.fff0c08p-1"),
    hf64!("0x1.ffef5d8p-1"),
    hf64!("0x1.ffedfa8p-1"),
    hf64!("0x1.ffec98p-1"),
    hf64!("0x1.ffeb35p-1"),
];

#[rustfmt::skip]
static LL: [[(f64, f64, f64); 17]; 4] = [
    [
        (hf64!("0x0p+0"), hf64!("0x0p+0"), hf64!("0x0p+0")),
        (hf64!("0x1.62e432b24p-6"), hf64!("-0x1.745af34bb54b8p-42"), hf64!("-0x1.17e3ec05cde7p-97")),
        (hf64!("0x1.62e42e4a8p-5"), hf64!("0x1.111a4eadf312p-44"), hf64!("0x1.cff3027abb119p-93")),
        (hf64!("0x1.0a2b233f1p-4"), hf64!("-0x1.88ac4ec78af8p-42"), hf64!("0x1.4fa087ca75dfdp-93")),
        (hf64!("0x1.62e43056cp-4"), hf64!("0x1.6bd65e8b0b7p-46"), hf64!("-0x1.b18e160362c24p-95")),
        (hf64!("0x1.bb9d3cbd6p-4"), hf64!("0x1.de14aa55ec2bp-42"), hf64!("-0x1.c6ac3f1862a6bp-94")),
        (hf64!("0x1.0a2b244dap-3"), hf64!("0x1.94def487fea7p-42"), hf64!("-0x1.dead1a4581acfp-94")),
        (hf64!("0x1.3687aa9b78p-3"), hf64!("0x1.9cec9a50db22p-43"), hf64!("0x1.34a70684f8e0ep-93")),
        (hf64!("0x1.62e42fabap-3"), hf64!("-0x1.d69047a3aebp-44"), hf64!("-0x1.4e061f79144e2p-95")),
        (hf64!("0x1.8f40b56d28p-3"), hf64!("0x1.de7d755fd2e2p-42"), hf64!("0x1.bdc7ecf001489p-94")),
        (hf64!("0x1.bb9d3b61fp-3"), hf64!("0x1.c14f1445b12p-46"), hf64!("0x1.a1d78cbdc5b58p-93")),
        (hf64!("0x1.e7f9c11f08p-3"), hf64!("-0x1.6e3e0000dae7p-43"), hf64!("0x1.6a4559fadde98p-94")),
        (hf64!("0x1.0a2b242ec4p-2"), hf64!("0x1.bb7cf852a5fe8p-42"), hf64!("0x1.a6aef11ee43bdp-93")),
        (hf64!("0x1.205966c764p-2"), hf64!("0x1.ad3a5f214294p-45"), hf64!("0x1.5cc344fa10652p-93")),
        (hf64!("0x1.3687a98aacp-2"), hf64!("0x1.1623671842fp-45"), hf64!("-0x1.0b428fe1f9e43p-94")),
        (hf64!("0x1.4cb5ec93f4p-2"), hf64!("0x1.3d50980ea513p-42"), hf64!("0x1.67f0ea083b1c4p-93")),
        (hf64!("0x1.62e42fefa4p-2"), hf64!("-0x1.8432a1b0e264p-44"), hf64!("0x1.803f2f6af40f3p-93")),
    ],
    [
        (hf64!("0x0p+0"), hf64!("0x0p+0"), hf64!("0x0p+0")),
        (hf64!("0x1.62e462b4p-10"), hf64!("0x1.061d003b97318p-42"), hf64!("0x1.d7faee66a2e1ep-93")),
        (hf64!("0x1.62e44c92p-9"), hf64!("0x1.95a7bff5e239p-42"), hf64!("-0x1.f7e788a87135p-95")),
        (hf64!("0x1.0a2b1e33p-8"), hf64!("0x1.2a3a1a65aa3ap-43"), hf64!("-0x1.54599c9605442p-93")),
        (hf64!("0x1.62e4367cp-8"), hf64!("-0x1.4a995b6d9ddcp-45"), hf64!("-0x1.56bb79b254f33p-100")),
        (hf64!("0x1.bb9d449ap-8"), hf64!("0x1.8a119c42e9bcp-42"), hf64!("-0x1.8ecf7d8d661f1p-93")),
        (hf64!("0x1.0a2b1f19p-7"), hf64!("0x1.8863771bd10a8p-42"), hf64!("0x1.e9731de7f0155p-94")),
        (hf64!("0x1.3687ad11p-7"), hf64!("0x1.e026a347ca1c8p-42"), hf64!("0x1.fadc62522444dp-97")),
        (hf64!("0x1.62e436f28p-7"), hf64!("0x1.25b84f71b70b8p-42"), hf64!("-0x1.fcb3f98612d27p-96")),
        (hf64!("0x1.8f40b7b38p-7"), hf64!("-0x1.62a0a4fd4758p-43"), hf64!("0x1.3cb3c35d9f6a1p-93")),
        (hf64!("0x1.bb9d3abbp-7"), hf64!("-0x1.0ec48f94d786p-42"), hf64!("-0x1.6b47d410e4cc7p-93")),
        (hf64!("0x1.e7f9bb23p-7"), hf64!("0x1.e4415cbc97ap-43"), hf64!("-0x1.3729fdb677231p-93")),
        (hf64!("0x1.0a2b22478p-6"), hf64!("-0x1.cb73f4505b03p-42"), hf64!("-0x1.1b3b3a3bc370ap-93")),
        (hf64!("0x1.2059691e8p-6"), hf64!("-0x1.abcc3412f264p-43"), hf64!("-0x1.fe6e998e48673p-95")),
        (hf64!("0x1.3687a768p-6"), hf64!("-0x1.43901e5c97a9p-42"), hf64!("0x1.b54cdd52a5d88p-96")),
        (hf64!("0x1.4cb5eb5d8p-6"), hf64!("-0x1.8f106f00f13b8p-42"), hf64!("-0x1.8f793f5fce148p-93")),
        (hf64!("0x1.62e432b24p-6"), hf64!("-0x1.745af34bb54b8p-42"), hf64!("-0x1.17e3ec05cde7p-97")),
    ],
    [
        (hf64!("0x0p+0"), hf64!("0x0p+0"), hf64!("0x0p+0")),
        (hf64!("0x1.62e7bp-14"), hf64!("-0x1.868625640a68p-44"), hf64!("-0x1.34bf0db910f65p-93")),
        (hf64!("0x1.62e35f6p-13"), hf64!("-0x1.2ee3d96b696ap-43"), hf64!("0x1.a2948cd558655p-94")),
        (hf64!("0x1.0a2b4b2p-12"), hf64!("0x1.53edbcf1165p-47"), hf64!("-0x1.cfc26ccf6d0e4p-97")),
        (hf64!("0x1.62e4be1p-12"), hf64!("0x1.783e334614p-52"), hf64!("-0x1.04b96da30e63ap-93")),
        (hf64!("0x1.bb9e085p-12"), hf64!("-0x1.60785f20acb2p-43"), hf64!("-0x1.f33369bf7dff1p-96")),
        (hf64!("0x1.0a2b94dp-11"), hf64!("0x1.fd4b3a273353p-42"), hf64!("-0x1.685a35575eff1p-96")),
        (hf64!("0x1.368810f8p-11"), hf64!("0x1.7ded26dc813p-47"), hf64!("-0x1.4c4d1abca79bfp-96")),
        (hf64!("0x1.62e47878p-11"), hf64!("0x1.7d2bee9a1f63p-42"), hf64!("0x1.860233b7ad13p-93")),
        (hf64!("0x1.8f40cb48p-11"), hf64!("-0x1.af034eaf471cp-42"), hf64!("0x1.ae748822d57b7p-94")),
        (hf64!("0x1.bb9d094p-11"), hf64!("-0x1.7a223013a20fp-42"), hf64!("-0x1.1e499087075b6p-93")),
        (hf64!("0x1.e7fa32c8p-11"), hf64!("-0x1.b2e67b1b59bdp-43"), hf64!("-0x1.54a41eda30fa6p-93")),
        (hf64!("0x1.0a2b237p-10"), hf64!("-0x1.7ad97ff4ac7ap-44"), hf64!("0x1.f932da91371ddp-93")),
        (hf64!("0x1.2059a338p-10"), hf64!("-0x1.96422d90df4p-44"), hf64!("-0x1.90800fbbf2ed3p-94")),
        (hf64!("0x1.36879824p-10"), hf64!("0x1.0f9054001812p-44"), hf64!("0x1.9567e01e48f9ap-93")),
        (hf64!("0x1.4cb602cp-10"), hf64!("-0x1.0d709a5ec0b5p-43"), hf64!("0x1.253dfd44635d2p-94")),
        (hf64!("0x1.62e462b4p-10"), hf64!("0x1.061d003b97318p-42"), hf64!("0x1.d7faee66a2e1ep-93")),
    ],
    [
        (hf64!("0x0p+0"), hf64!("0x0p+0"), hf64!("0x0p+0")),
        (hf64!("0x1.63007cp-18"), hf64!("-0x1.db0e38e5aaaap-43"), hf64!("0x1.259a7b94815b9p-93")),
        (hf64!("0x1.6300f6p-17"), hf64!("0x1.2b1c75580438p-44"), hf64!("0x1.78cabba01e3e4p-93")),
        (hf64!("0x1.0a2115p-16"), hf64!("-0x1.5ff223730759p-42"), hf64!("0x1.8074feacfe49dp-95")),
        (hf64!("0x1.62e1ecp-16"), hf64!("-0x1.85d6f6487ce4p-45"), hf64!("0x1.05485074b9276p-93")),
        (hf64!("0x1.bba301p-16"), hf64!("-0x1.af5d58a7c921p-43"), hf64!("-0x1.30a8c0fd2ff5fp-93")),
        (hf64!("0x1.0a32298p-15"), hf64!("0x1.590faa0883bdp-43"), hf64!("0x1.95e9bda999947p-93")),
        (hf64!("0x1.3682f1p-15"), hf64!("0x1.f0224376efaf8p-42"), hf64!("-0x1.5843c0db50d1p-93")),
        (hf64!("0x1.62e3d8p-15"), hf64!("-0x1.142c13daed4ap-43"), hf64!("0x1.c68a61183ce87p-93")),
        (hf64!("0x1.8f44dd8p-15"), hf64!("-0x1.aa489f399931p-43"), hf64!("0x1.11c5c376854eap-94")),
        (hf64!("0x1.bb9601p-15"), hf64!("0x1.9904d8b6a3638p-42"), hf64!("0x1.8c89554493c8fp-93")),
        (hf64!("0x1.e7f744p-15"), hf64!("0x1.5785ddbe7cba8p-42"), hf64!("0x1.e7ff3cde7d70cp-94")),
        (hf64!("0x1.0a2c53p-14"), hf64!("-0x1.6d9e8780d0d5p-43"), hf64!("0x1.ad9c178106693p-94")),
        (hf64!("0x1.205d134p-14"), hf64!("-0x1.214a2e893fccp-43"), hf64!("0x1.548a9500c9822p-93")),
        (hf64!("0x1.3685e28p-14"), hf64!("0x1.e23588646103p-43"), hf64!("0x1.2a97b26da2d88p-94")),
        (hf64!("0x1.4cb6c18p-14"), hf64!("0x1.2b7cfcea9e0d8p-42"), hf64!("-0x1.5095048a6b824p-93")),
        (hf64!("0x1.62e7bp-14"), hf64!("-0x1.868625640a68p-44"), hf64!("-0x1.34bf0db910f65p-93")),
    ],
];
static CH: &[(f64, f64)] = &[
    (hf64!("0x1p-1"), hf64!("0x1.24b67ee516e3bp-111")),
    (hf64!("-0x1p-2"), hf64!("-0x1.932ce43199a8dp-110")),
    (
        hf64!("0x1.5555555555555p-3"),
        hf64!("0x1.55540c15cf91fp-57"),
    ),
];
static CL: [f64; 3] = [
    hf64!("-0x1p-3"),
    hf64!("0x1.9999999a0754fp-4"),
    hf64!("-0x1.55555555c3157p-4"),
];

fn as_acosh_refine(x: f64, a: f64) -> f64 {
    let ix = x.to_bits();
    let mut zh: f64;
    let mut zl: f64;
    if ix < 0x4190000000000000_u64 {
        let x2h = x * x;
        let x2l = x.fma(x, -x2h);
        let wl;
        let mut wh = x2h - 1.0;
        (wh, wl) = fasttwosum(wh, x2l);
        let sh = wh.sqrt();
        let ish = 0.5 / wh;
        let sl = (ish * sh) * (wl - sh.fma(sh, -wh));
        (zh, zl) = fasttwosum(x, sh);
        zl += sl;
        (zh, zl) = fasttwosum(zh, zl);
    } else if ix < 0x4330000000000000_u64 {
        zh = 2.0 * x;
        zl = -0.5 / x;
    } else {
        zh = x;
        zl = 0.0;
    }
    let mut t = zh.to_bits();
    let ex: i32 = (t >> 52).cast();
    let e: i32 = ex - 0x3ff + (zl == 0.0) as i32;
    t &= u64::MAX >> 12;
    t |= 0x3ff << 52;
    let ed: f64 = e.cast();
    let v = (a - ed + hf64!("0x1.00008p+0")).to_bits();
    let i: u64 = (v - (0x3ff_u64 << 52)) >> (52 - 16);
    let i1: i32 = ((i >> 12) & 0x1f).cast();
    let i2: i32 = ((i >> 8) & 0xf).cast();
    let i3: i32 = ((i >> 4) & 0xf).cast();
    let i4: i32 = (i & 0xf).cast();
    let l20 = hf64!("0x1.62e42fefa38p-2");
    let l21 = hf64!("0x1.ef35793c768p-46");
    let l22 = hf64!("-0x1.9ff0342542fc3p-91");
    let el2: f64 = l22 * ed;
    let el1: f64 = l21 * ed;
    let el0: f64 = l20 * ed;
    let mut l = [0.0; 3];
    l[0] =
        LL[0][i1 as usize].0 + LL[1][i2 as usize].0 + (LL[2][i3 as usize].0 + LL[3][i4 as usize].0);
    l[1] =
        LL[0][i1 as usize].1 + LL[1][i2 as usize].1 + (LL[2][i3 as usize].1 + LL[3][i4 as usize].1);
    l[2] =
        LL[0][i1 as usize].2 + LL[1][i2 as usize].2 + (LL[2][i3 as usize].2 + LL[3][i4 as usize].2);
    l[0] += el0;
    let t12 = T1[i1 as usize] * T2[i2 as usize];
    let t34 = T3[i3 as usize] * T4[i4 as usize];
    let th = t12 * t34;
    let tl = t12.fma(t34, -th);
    let dh = th * f64::from_bits(t);
    let dl = th.fma(f64::from_bits(t), -dh);
    let mut sh = tl * f64::from_bits(t);
    let mut sl = tl.fma(f64::from_bits(t), -sh);
    let (mut xh, mut xl) = fasttwosum(dh - 1.0, dl);
    if zl != 0.0 {
        t = zl.to_bits();
        t -= ((e as i64) << 52).unsigned();
        xl += th * f64::from_bits(t);
    }
    (xh, xl) = adddd(xh, xl, sh, sl);
    sl = xh * (CL[0] + xh * (CL[1] + xh * CL[2]));
    (sh, sl) = polydd(xh, xl, 3, CH, sl);
    (sh, sl) = muldd_acc(xh, xl, sh, sl);
    (sh, sl) = adddd(sh, sl, el1, el2);
    (sh, sl) = adddd(sh, sl, l[1], l[2]);
    let (mut v0, v2) = fasttwosum(l[0], sh);
    let (mut v1, mut v2) = fasttwosum(v2, sl);
    v0 *= 2.0;
    v1 *= 2.0;
    v2 *= 2.0;
    t = v1.cast();
    if t & (u64::MAX >> 12) == 0 {
        cold_path();
        let w = v2.to_bits();
        if ((w ^ t) >> 63) != 0 {
            t -= 1;
        } else {
            t += 1;
        }
        v1 = f64::from_bits(t);
    }
    let t0 = v0.to_bits();
    let er = (t + 7) & (u64::MAX >> 12);
    let de = ((t0 >> 52) & 0x7ff) - ((t >> 52) & 0x7ff);
    let res = v0 + v1;
    if de > 102 || er < 15 {
        cold_path();
        return as_acosh_database(x, res);
    }

    res
}
