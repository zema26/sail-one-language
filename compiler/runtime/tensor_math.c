/* Appended to runtime.c when building either target. All tensors are row-major,
   region-owned allocations; returned slices and arithmetic results are copies. */
#ifdef __wasm__
IMPORT("exp") double host_exp(double);
static double tensor_exp_scalar(double x) { return host_exp(x); }
#else
#include <math.h>
static double tensor_exp_scalar(double x) { return exp(x); }
#endif

typedef union { double f; i64 i; } TensorCell;
static double tensor_number(i64 bits, i64 floating) {
    TensorCell v; v.i = bits;
    return floating ? v.f : (double)bits;
}
static i64 tensor_bits(double number) { TensorCell v; v.f = number; return v.i; }
static i64 *tensor_cells(i64 *v) { return v + 2 + v[1]; }

void sail_tensor_check(i64 *v, i64 rank, const i64 *shape) {
    if (!v || v[1] != rank) fail(7);
    for (i64 i = 0; i < rank; i++) if (shape[i] < 0 || v[2+i] != shape[i]) fail(7);
}
void sail_tensor_same_shape(i64 *a, i64 *b) {
    if (!a || !b) fail(7);
    sail_tensor_check(b, a[1], a+2);
}
void *sail_tensor_slice(i64 *v, i64 count, const i64 *indices) {
    if (!v || count < 1 || count >= v[1]) fail(2);
    i64 offset = 0;
    for (i64 i = 0; i < count; i++) {
        if (indices[i] < 0 || indices[i] >= v[2+i]) fail(2);
        offset = offset * v[2+i] + indices[i];
    }
    i64 *r = sail_tensor_new(v[1]-count, v+2+count);
    i64 *src = tensor_cells(v), *dest = tensor_cells(r);
    offset *= r[0];
    for (i64 i = 0; i < r[0]; i++) dest[i] = src[offset+i];
    return r;
}
static i64 tensor_calculate(i64 a, i64 b, i64 ak, i64 bk, i64 op) {
    if (ak || bk) {
        double x = tensor_number(a, ak), y = tensor_number(b, bk);
        return tensor_bits(op==0 ? x+y : op==1 ? x-y : op==2 ? x*y : x/y);
    }
    if (op==3) return sail_idiv(a,b);
    /* Sail's LLVM integer arithmetic wraps, without invoking C signed UB. */
    return (i64)(op==0 ? (u64)a+(u64)b : op==1 ? (u64)a-(u64)b : (u64)a*(u64)b);
}
void *sail_tensor_binary(i64 *a, i64 *b, i64 op, i64 ak, i64 bk) {
    sail_tensor_same_shape(a,b);
    i64 *r = sail_tensor_new(a[1], a+2);
    i64 *x = tensor_cells(a), *y = tensor_cells(b), *z = tensor_cells(r);
    for (i64 i = 0; i < a[0]; i++) z[i] = tensor_calculate(x[i],y[i],ak,bk,op);
    return r;
}
void *sail_tensor_scalar(i64 *a, i64 scalar, i64 op, i64 ak, i64 sk, i64 reverse) {
    i64 *r = sail_tensor_new(a[1], a+2), *x = tensor_cells(a), *z = tensor_cells(r);
    for (i64 i = 0; i < a[0]; i++)
        z[i] = reverse ? tensor_calculate(scalar,x[i],sk,ak,op) : tensor_calculate(x[i],scalar,ak,sk,op);
    return r;
}
void *sail_tensor_exp(i64 *a, i64 kind) {
    i64 *r = sail_tensor_new(a[1], a+2), *x = tensor_cells(a), *z = tensor_cells(r);
    for (i64 i = 0; i < a[0]; i++) z[i] = tensor_bits(tensor_exp_scalar(tensor_number(x[i],kind)));
    return r;
}
/* ClairII's cross means outer product, not the three-dimensional vector cross. */
void *sail_tensor_outer(i64 *a, i64 *b, i64 ak, i64 bk) {
    if (a[1] != 1 || b[1] != 1) fail(7);
    i64 shape[2] = {a[2], b[2]};
    i64 *r = sail_tensor_new(2, shape);
    i64 *x = tensor_cells(a), *y = tensor_cells(b), *z = tensor_cells(r);
    for (i64 i=0; i<shape[0]; i++) for (i64 j=0; j<shape[1]; j++)
        z[i*shape[1]+j] = tensor_calculate(x[i],y[j],ak,bk,2);
    return r;
}
i64 sail_tensor_reduce(i64 *a, i64 kind, i64 op) {
    i64 *x = tensor_cells(a);
    if (op==0) {
        if (kind) {
            double sum = 0;
            for (i64 i = 0; i < a[0]; i++) sum += tensor_number(x[i],1);
            return tensor_bits(sum);
        }
        u64 sum = 0; for (i64 i = 0; i < a[0]; i++) sum += (u64)x[i];
        return (i64)sum;
    }
    if (!a[0]) fail(8);
    i64 best = 0;
    for (i64 i = 0; i < a[0]; i++) {
        if (kind) {
            double v = tensor_number(x[i],1);
            if (v != v) fail(9);
            if (v > tensor_number(x[best],1)) best = i;
        } else if (x[i] > x[best]) best = i;
    }
    return op==2 ? best : x[best];
}
void *sail_tensor_matmul(i64 *a, i64 *b, i64 ak, i64 bk) {
    if (a[1]>2 || b[1]>2) fail(7);
    i64 m = a[1]==1 ? 1 : a[2], k = a[1]==1 ? a[2] : a[3];
    i64 kb = b[2], n = b[1]==1 ? 1 : b[3];
    if (k != kb) fail(7);
    if (m*n*k > 50000000) fail(10);
    i64 shape[2] = {m,n}, rank = 2;
    if (a[1]==1 || b[1]==1) { rank=1; shape[0] = a[1]==1 ? n : m; }
    i64 *r = sail_tensor_new(rank,shape), *x = tensor_cells(a), *y = tensor_cells(b), *z = tensor_cells(r);
    for (i64 i = 0; i < m; i++) for (i64 j = 0; j < n; j++) {
        if (ak || bk) {
            double sum = 0;
            for (i64 p = 0; p < k; p++) sum += tensor_number(x[i*k+p],ak)*tensor_number(y[p*n+j],bk);
            z[i*n+j] = tensor_bits(sum);
        } else {
            u64 sum = 0;
            for (i64 p = 0; p < k; p++) sum += (u64)x[i*k+p]*(u64)y[p*n+j];
            z[i*n+j] = (i64)sum;
        }
    }
    return r;
}
