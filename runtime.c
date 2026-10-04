/* Sail runtime. No filesystem or libc is exposed by the WebAssembly target. */
typedef long long i64;
typedef unsigned long long u64;
typedef unsigned int u32;
typedef _Bool boolean;

#ifdef __wasm__
#define IMPORT(name) __attribute__((import_module("sail"), import_name(name)))
IMPORT("print_int") void sail_print_int(i64);
IMPORT("print_float") void sail_print_float(double);
IMPORT("print_bool") void sail_print_bool(boolean);
IMPORT("print_char") void sail_print_char(i64);
IMPORT("print_string") void sail_print_string(const char *);
IMPORT("read_int") i64 sail_read_int(void);
IMPORT("read_float") double sail_read_float(void);
IMPORT("read_bool") boolean sail_read_bool(void);
IMPORT("read_char") i64 sail_read_char(void);
IMPORT("read_string") char *sail_read_string(void);
IMPORT("panic") void host_panic(i64);
IMPORT("file_new") void *sail_file_new(const char *, const char *);
IMPORT("file_open") void sail_file_open(void *);
IMPORT("file_close") void sail_file_close(void *);
IMPORT("file_eof") boolean sail_file_eof(void *);
IMPORT("file_read") char *sail_file_read(void *);
IMPORT("file_write") void sail_file_write(void *, const char *);
static unsigned char heap[8 * 1024 * 1024];
static u32 used;
static void fail(i64 code) { host_panic(code); __builtin_trap(); }
__attribute__((export_name("sail_alloc")))
void *sail_alloc(i64 n) {
    if (n < 0 || n > (i64)sizeof(heap)) fail(1);
    u32 bytes = ((u32)n + 7u) & ~7u;
    if (used > sizeof(heap) - bytes) fail(1);
    void *ptr = heap + used;
    used += bytes;
    return ptr;
}
#else
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <limits.h>
static void fail(i64 code) {
    const char *messages[] = {"runtime failure","allocation limit exceeded","vector index out of bounds","integer division by zero or overflow","invalid character","file operation failed","invalid input"};
    fprintf(stderr, "Sail runtime error: %s\n", messages[code >= 0 && code < 7 ? code : 0]);
    exit(70);
}
/* Region-owned allocations are released when the process ends. */
static u64 allocated;
void *sail_alloc(i64 n) {
    if (n < 0 || n > 8*1024*1024 || allocated + (u64)n > 8*1024*1024) fail(1);
    void *p = calloc(1, n ? (size_t)n : 1);
    if (!p) fail(1);
    allocated += (u64)n;
    return p;
}
void sail_print_int(i64 x) { printf("%lld",x); }
void sail_print_float(double x) { printf("%.12g",x); }
void sail_print_bool(boolean x) { fputs(x ? "true" : "false", stdout); }
void sail_print_char(i64 x) {
    if (x < 0 || x > 0x10ffff || (x >= 0xd800 && x <= 0xdfff)) fail(4);
    if (x < 128) putchar((int)x);
    else if (x < 2048) { putchar(0xc0 | (x >> 6)); putchar(0x80 | (x & 63)); }
    else if (x < 65536) { putchar(0xe0 | (x >> 12)); putchar(0x80 | ((x >> 6) & 63)); putchar(0x80 | (x & 63)); }
    else { putchar(0xf0 | (x >> 18)); putchar(0x80 | ((x >> 12) & 63)); putchar(0x80 | ((x >> 6) & 63)); putchar(0x80 | (x & 63)); }
}
void sail_print_string(const char *x) { fputs(x,stdout); }
char *sail_read_string(void) {
    char tmp[65536];
    if (scanf("%65535s",tmp)!=1) fail(6);
    size_t n=strlen(tmp)+1;
    char *p=sail_alloc((i64)n);memcpy(p,tmp,n);return p;
}
i64 sail_read_int(void) {
    char *s=sail_read_string(), *end=0;
    extern int *__errno_location(void);
    *__errno_location()=0;
    i64 n=strtoll(s,&end,10);
    if (*end || end==s || *__errno_location()) fail(6);
    return n;
}
double sail_read_float(void) {
    char *s=sail_read_string(), *end=0;
    double n=strtod(s,&end);
    if (*end || end==s || !__builtin_isfinite(n)) fail(6);
    return n;
}
boolean sail_read_bool(void) {
    char *s=sail_read_string();
    if (!strcmp(s,"true")||!strcmp(s,"1")) return 1;
    if (!strcmp(s,"false")||!strcmp(s,"0")) return 0;
    fail(6);return 0;
}
i64 sail_to_char(const char *);
i64 sail_read_char(void) { return sail_to_char(sail_read_string()); }
typedef struct { const char *path; const char *mode; FILE *handle; boolean eof; boolean opened; } SailFile;
void *sail_file_new(const char *path,const char *mode) {
    SailFile *f=sail_alloc(sizeof(SailFile));f->path=path;f->mode=mode;return f;
}
void sail_file_open(void *p) {
    SailFile *f=p;if (!f || f->handle) fail(5);
    const char *mode;
    if (strchr(f->mode,'a')) mode="a+";
    else if (strchr(f->mode,'w') && !f->opened) mode=strchr(f->mode,'r') || strchr(f->mode,'+') ? "w+" : "w";
    else mode=strchr(f->mode,'w') || strchr(f->mode,'+') ? "r+" : "r";
    f->handle=fopen(f->path,mode);if(!f->handle)fail(5);f->eof=0;f->opened=1;
}
void sail_file_close(void *p) { SailFile *f=p;if (!f || !f->handle)fail(5);if(fclose(f->handle))fail(5);f->handle=0; }
boolean sail_file_eof(void *p) {
    SailFile *f=p;if (!f || !f->handle)fail(5);
    int c=fgetc(f->handle);
    if(c==EOF){if(ferror(f->handle))fail(5);f->eof=1;return 1;}
    ungetc(c,f->handle);return 0;
}
char *sail_file_read(void *p) {
    SailFile *f=p;if(!f || !f->handle)fail(5);
    char tmp[65536];size_t n=0;int c;
    while((c=fgetc(f->handle))!=EOF && c!='\n'){if(n>=sizeof(tmp)-1)fail(5);tmp[n++]=(char)c;}
    if(ferror(f->handle))fail(5);tmp[n]=0;
    char *s=sail_alloc((i64)n+1);memcpy(s,tmp,n+1);return s;
}
void sail_file_write(void *p,const char *s) {
    SailFile *f=p;if(!f || !f->handle || (!strchr(f->mode,'w') && !strchr(f->mode,'a')))fail(5);
    if(fputs(s,f->handle)==EOF)fail(5);
}
extern i64 sail_main(void);
int main(void) { return (int)sail_main(); }
#endif

i64 sail_strlen(const char *s) { i64 n=0;while(s[n])n++;return n; }
void *sail_vec_new(i64 n) {
    if (n<0 || n>500000) fail(1);
    i64 *v=sail_alloc(8+n*8);v[0]=n;return v;
}
void *sail_vec_at(i64 *v,i64 index) {
    if(!v || index<0 || index>=v[0])fail(2);
    return &v[index+1];
}
i64 sail_string_at(const char *s,i64 index) {
    if(index<0 || index>=sail_strlen(s))fail(2);
    return (unsigned char)s[index];
}
i64 sail_to_char(const char *s) {
    const unsigned char *p=(const unsigned char *)s;
    i64 n=p[0];int size=1;
    if(n>=0xc2 && n<=0xdf){n&=31;size=2;}
    else if(n>=0xe0 && n<=0xef){n&=15;size=3;}
    else if(n>=0xf0 && n<=0xf4){n&=7;size=4;}
    else if(n>=128)fail(4);
    for(int i=1;i<size;i++){if((p[i]&0xc0)!=0x80)fail(4);n=(n<<6)|(p[i]&63);}
    if(p[size] || !p[0] || n>0x10ffff || (n>=0xd800 && n<=0xdfff))fail(4);
    return n;
}
char *sail_concat(const char *a,const char *b) {
    i64 n=sail_strlen(a),m=sail_strlen(b);
    char *s=sail_alloc(n+m+1);
    for(i64 i=0;i<n;i++)s[i]=a[i];
    for(i64 i=0;i<m;i++)s[n+i]=b[i];
    s[n+m]=0;return s;
}
boolean sail_str_eq(const char *a,const char *b) {
    i64 i=0;while(a[i] && a[i]==b[i])i++;return a[i]==b[i];
}
i64 sail_idiv(i64 a,i64 b) {
    if(!b || (a==(-9223372036854775807LL-1) && b==-1))fail(3);
    return a/b;
}
i64 sail_imod(i64 a,i64 b) {
    if(!b || (a==(-9223372036854775807LL-1) && b==-1))fail(3);
    return a%b;
}