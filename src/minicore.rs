// Minimal core runtime for compiling pure Rust directly to wasm32-unknown-unknown
// without external rlib dependencies.

#[lang = "pointee_sized"]
pub trait PointeeSized {}

#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}

#[lang = "sized"]
pub trait Sized: MetaSized {}

#[lang = "copy"]
pub trait Copy {}

#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[lang = "sync"]
pub unsafe auto trait Sync {}

impl Copy for bool {}
impl Copy for u8 {}
impl Copy for i8 {}
impl Copy for u16 {}
impl Copy for i16 {}
impl Copy for u32 {}
impl Copy for i32 {}
impl Copy for u64 {}
impl Copy for i64 {}
impl Copy for usize {}
impl Copy for isize {}
impl Copy for f32 {}
impl Copy for f64 {}
impl<T: ?Sized> Copy for *const T {}
impl<T: ?Sized> Copy for *mut T {}
impl<T: ?Sized> Copy for &T {}

#[lang = "legacy_receiver"]
pub trait LegacyReceiver {}
impl<T: ?Sized> LegacyReceiver for &T {}
impl<T: ?Sized> LegacyReceiver for &mut T {}
impl<T: ?Sized> LegacyReceiver for *const T {}
impl<T: ?Sized> LegacyReceiver for *mut T {}

#[lang = "receiver"]
pub trait Receiver {}
impl<T: ?Sized> Receiver for &T {}
impl<T: ?Sized> Receiver for &mut T {}

#[rustc_intrinsic]
pub fn sqrtf32(x: f32) -> f32;
#[rustc_intrinsic]
pub fn sinf32(x: f32) -> f32;
#[rustc_intrinsic]
pub fn cosf32(x: f32) -> f32;
#[rustc_intrinsic]
pub fn floorf32(x: f32) -> f32;
#[rustc_intrinsic]
pub const unsafe fn arith_offset<T>(dst: *const T, offset: isize) -> *const T;

#[inline(always)]
pub fn absf32(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

#[lang = "add"]
pub trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}
#[lang = "sub"]
pub trait Sub<Rhs = Self> {
    type Output;
    fn sub(self, rhs: Rhs) -> Self::Output;
}
#[lang = "mul"]
pub trait Mul<Rhs = Self> {
    type Output;
    fn mul(self, rhs: Rhs) -> Self::Output;
}
#[lang = "div"]
pub trait Div<Rhs = Self> {
    type Output;
    fn div(self, rhs: Rhs) -> Self::Output;
}
#[lang = "rem"]
pub trait Rem<Rhs = Self> {
    type Output;
    fn rem(self, rhs: Rhs) -> Self::Output;
}
#[lang = "neg"]
pub trait Neg {
    type Output;
    fn neg(self) -> Self::Output;
}
#[lang = "not"]
pub trait Not {
    type Output;
    fn not(self) -> Self::Output;
}
#[lang = "bitand"]
pub trait BitAnd<Rhs = Self> {
    type Output;
    fn bitand(self, rhs: Rhs) -> Self::Output;
}
#[lang = "bitor"]
pub trait BitOr<Rhs = Self> {
    type Output;
    fn bitor(self, rhs: Rhs) -> Self::Output;
}
#[lang = "bitxor"]
pub trait BitXor<Rhs = Self> {
    type Output;
    fn bitxor(self, rhs: Rhs) -> Self::Output;
}
#[lang = "shl"]
pub trait Shl<Rhs = Self> {
    type Output;
    fn shl(self, rhs: Rhs) -> Self::Output;
}
#[lang = "shr"]
pub trait Shr<Rhs = Self> {
    type Output;
    fn shr(self, rhs: Rhs) -> Self::Output;
}
#[lang = "add_assign"]
pub trait AddAssign<Rhs = Self> {
    fn add_assign(&mut self, rhs: Rhs);
}
#[lang = "sub_assign"]
pub trait SubAssign<Rhs = Self> {
    fn sub_assign(&mut self, rhs: Rhs);
}
#[lang = "mul_assign"]
pub trait MulAssign<Rhs = Self> {
    fn mul_assign(&mut self, rhs: Rhs);
}
#[lang = "bitand_assign"]
pub trait BitAndAssign<Rhs = Self> {
    fn bitand_assign(&mut self, rhs: Rhs);
}
#[lang = "bitor_assign"]
pub trait BitOrAssign<Rhs = Self> {
    fn bitor_assign(&mut self, rhs: Rhs);
}

#[lang = "eq"]
pub trait PartialEq<Rhs: ?Sized = Self> {
    fn eq(&self, other: &Rhs) -> bool;
    fn ne(&self, other: &Rhs) -> bool {
        !self.eq(other)
    }
}

#[lang = "partial_ord"]
pub trait PartialOrd<Rhs: ?Sized = Self> {
    fn lt(&self, other: &Rhs) -> bool;
    fn le(&self, other: &Rhs) -> bool;
    fn gt(&self, other: &Rhs) -> bool;
    fn ge(&self, other: &Rhs) -> bool;
}

impl Not for bool {
    type Output = bool;
    fn not(self) -> bool {
        if self { false } else { true }
    }
}
impl PartialEq for bool {
    fn eq(&self, other: &bool) -> bool {
        *self == *other
    }
}

impl Add for i32 { type Output = i32; fn add(self, r: i32) -> i32 { self + r } }
impl Sub for i32 { type Output = i32; fn sub(self, r: i32) -> i32 { self - r } }
impl Mul for i32 { type Output = i32; fn mul(self, r: i32) -> i32 { self * r } }
impl Div for i32 { type Output = i32; fn div(self, r: i32) -> i32 { self / r } }
impl Rem for i32 { type Output = i32; fn rem(self, r: i32) -> i32 { self % r } }
impl Neg for i32 { type Output = i32; fn neg(self) -> i32 { -self } }
impl Not for i32 { type Output = i32; fn not(self) -> i32 { !self } }
impl BitAnd for i32 { type Output = i32; fn bitand(self, r: i32) -> i32 { self & r } }
impl BitOr for i32 { type Output = i32; fn bitor(self, r: i32) -> i32 { self | r } }
impl BitXor for i32 { type Output = i32; fn bitxor(self, r: i32) -> i32 { self ^ r } }
impl Shl<i32> for i32 { type Output = i32; fn shl(self, r: i32) -> i32 { self << r } }
impl Shr<i32> for i32 { type Output = i32; fn shr(self, r: i32) -> i32 { self >> r } }
impl Shl<u32> for i32 { type Output = i32; fn shl(self, r: u32) -> i32 { self << r } }
impl Shr<u32> for i32 { type Output = i32; fn shr(self, r: u32) -> i32 { self >> r } }
impl AddAssign for i32 { fn add_assign(&mut self, r: i32) { *self = *self + r; } }
impl SubAssign for i32 { fn sub_assign(&mut self, r: i32) { *self = *self - r; } }
impl MulAssign for i32 { fn mul_assign(&mut self, r: i32) { *self = *self * r; } }
impl PartialEq for i32 { fn eq(&self, o: &i32) -> bool { *self == *o } }
impl PartialOrd for i32 {
    fn lt(&self, o: &i32) -> bool { *self < *o }
    fn le(&self, o: &i32) -> bool { *self <= *o }
    fn gt(&self, o: &i32) -> bool { *self > *o }
    fn ge(&self, o: &i32) -> bool { *self >= *o }
}

impl Add for u32 { type Output = u32; fn add(self, r: u32) -> u32 { self + r } }
impl Sub for u32 { type Output = u32; fn sub(self, r: u32) -> u32 { self - r } }
impl Mul for u32 { type Output = u32; fn mul(self, r: u32) -> u32 { self * r } }
impl Div for u32 { type Output = u32; fn div(self, r: u32) -> u32 { self / r } }
impl Rem for u32 { type Output = u32; fn rem(self, r: u32) -> u32 { self % r } }
impl Not for u32 { type Output = u32; fn not(self) -> u32 { !self } }
impl BitAnd for u32 { type Output = u32; fn bitand(self, r: u32) -> u32 { self & r } }
impl BitOr for u32 { type Output = u32; fn bitor(self, r: u32) -> u32 { self | r } }
impl BitXor for u32 { type Output = u32; fn bitxor(self, r: u32) -> u32 { self ^ r } }
impl Shl<u32> for u32 { type Output = u32; fn shl(self, r: u32) -> u32 { self << r } }
impl Shr<u32> for u32 { type Output = u32; fn shr(self, r: u32) -> u32 { self >> r } }
impl Shl<i32> for u32 { type Output = u32; fn shl(self, r: i32) -> u32 { self << r } }
impl Shr<i32> for u32 { type Output = u32; fn shr(self, r: i32) -> u32 { self >> r } }
impl AddAssign for u32 { fn add_assign(&mut self, r: u32) { *self = *self + r; } }
impl SubAssign for u32 { fn sub_assign(&mut self, r: u32) { *self = *self - r; } }
impl BitOrAssign for u32 { fn bitor_assign(&mut self, r: u32) { *self = *self | r; } }
impl BitAndAssign for u32 { fn bitand_assign(&mut self, r: u32) { *self = *self & r; } }
impl PartialEq for u32 { fn eq(&self, o: &u32) -> bool { *self == *o } }
impl PartialOrd for u32 {
    fn lt(&self, o: &u32) -> bool { *self < *o }
    fn le(&self, o: &u32) -> bool { *self <= *o }
    fn gt(&self, o: &u32) -> bool { *self > *o }
    fn ge(&self, o: &u32) -> bool { *self >= *o }
}

impl Add for u8 { type Output = u8; fn add(self, r: u8) -> u8 { self + r } }
impl Sub for u8 { type Output = u8; fn sub(self, r: u8) -> u8 { self - r } }
impl Mul for u8 { type Output = u8; fn mul(self, r: u8) -> u8 { self * r } }
impl BitAnd for u8 { type Output = u8; fn bitand(self, r: u8) -> u8 { self & r } }
impl BitOr for u8 { type Output = u8; fn bitor(self, r: u8) -> u8 { self | r } }
impl Shl<i32> for u8 { type Output = u8; fn shl(self, r: i32) -> u8 { self << r } }
impl Shr<i32> for u8 { type Output = u8; fn shr(self, r: i32) -> u8 { self >> r } }
impl Shl<u8> for u8 { type Output = u8; fn shl(self, r: u8) -> u8 { self << r } }
impl Shr<u8> for u8 { type Output = u8; fn shr(self, r: u8) -> u8 { self >> r } }
impl AddAssign for u8 { fn add_assign(&mut self, r: u8) { *self = *self + r; } }
impl SubAssign for u8 { fn sub_assign(&mut self, r: u8) { *self = *self - r; } }
impl PartialEq for u8 { fn eq(&self, o: &u8) -> bool { *self == *o } }
impl PartialOrd for u8 {
    fn lt(&self, o: &u8) -> bool { *self < *o }
    fn le(&self, o: &u8) -> bool { *self <= *o }
    fn gt(&self, o: &u8) -> bool { *self > *o }
    fn ge(&self, o: &u8) -> bool { *self >= *o }
}

impl<T: Copy, const N: usize> Copy for [T; N] {}

#[lang = "index"]
pub trait Index<Idx: ?Sized> {
    type Output: ?Sized;
    fn index(&self, index: Idx) -> &Self::Output;
}

impl<T, const N: usize> Index<usize> for [T; N] {
    type Output = T;
    #[inline(always)]
    fn index(&self, index: usize) -> &T {
        unsafe { &*arith_offset(self as *const [T; N] as *const T, index as isize) }
    }
}

pub trait PtrExt<T> {
    unsafe fn offset(self, count: isize) -> Self;
}

impl<T> PtrExt<T> for *const T {
    #[inline(always)]
    unsafe fn offset(self, count: isize) -> *const T {
        arith_offset(self, count)
    }
}

impl<T> PtrExt<T> for *mut T {
    #[inline(always)]
    unsafe fn offset(self, count: isize) -> *mut T {
        arith_offset(self as *const T, count) as *mut T
    }
}

impl Add for usize { type Output = usize; fn add(self, r: usize) -> usize { self + r } }
impl Sub for usize { type Output = usize; fn sub(self, r: usize) -> usize { self - r } }
impl Mul for usize { type Output = usize; fn mul(self, r: usize) -> usize { self * r } }
impl Div for usize { type Output = usize; fn div(self, r: usize) -> usize { self / r } }
impl Rem for usize { type Output = usize; fn rem(self, r: usize) -> usize { self % r } }
impl BitAnd for usize { type Output = usize; fn bitand(self, r: usize) -> usize { self & r } }
impl BitOr for usize { type Output = usize; fn bitor(self, r: usize) -> usize { self | r } }
impl Shl<usize> for usize { type Output = usize; fn shl(self, r: usize) -> usize { self << r } }
impl Shr<usize> for usize { type Output = usize; fn shr(self, r: usize) -> usize { self >> r } }
impl AddAssign for usize { fn add_assign(&mut self, r: usize) { *self = *self + r; } }
impl PartialEq for usize { fn eq(&self, o: &usize) -> bool { *self == *o } }
impl PartialOrd for usize {
    fn lt(&self, o: &usize) -> bool { *self < *o }
    fn le(&self, o: &usize) -> bool { *self <= *o }
    fn gt(&self, o: &usize) -> bool { *self > *o }
    fn ge(&self, o: &usize) -> bool { *self >= *o }
}

impl Add for isize { type Output = isize; fn add(self, r: isize) -> isize { self + r } }
impl Sub for isize { type Output = isize; fn sub(self, r: isize) -> isize { self - r } }
impl Mul for isize { type Output = isize; fn mul(self, r: isize) -> isize { self * r } }
impl AddAssign for isize { fn add_assign(&mut self, r: isize) { *self = *self + r; } }
impl PartialEq for isize { fn eq(&self, o: &isize) -> bool { *self == *o } }
impl PartialOrd for isize {
    fn lt(&self, o: &isize) -> bool { *self < *o }
    fn le(&self, o: &isize) -> bool { *self <= *o }
    fn gt(&self, o: &isize) -> bool { *self > *o }
    fn ge(&self, o: &isize) -> bool { *self >= *o }
}

impl Add for f32 { type Output = f32; fn add(self, r: f32) -> f32 { self + r } }
impl Sub for f32 { type Output = f32; fn sub(self, r: f32) -> f32 { self - r } }
impl Mul for f32 { type Output = f32; fn mul(self, r: f32) -> f32 { self * r } }
impl Div for f32 { type Output = f32; fn div(self, r: f32) -> f32 { self / r } }
impl Neg for f32 { type Output = f32; fn neg(self) -> f32 { -self } }
impl AddAssign for f32 { fn add_assign(&mut self, r: f32) { *self = *self + r; } }
impl SubAssign for f32 { fn sub_assign(&mut self, r: f32) { *self = *self - r; } }
impl MulAssign for f32 { fn mul_assign(&mut self, r: f32) { *self = *self * r; } }
impl PartialEq for f32 { fn eq(&self, o: &f32) -> bool { *self == *o } }
impl PartialOrd for f32 {
    fn lt(&self, o: &f32) -> bool { *self < *o }
    fn le(&self, o: &f32) -> bool { *self <= *o }
    fn gt(&self, o: &f32) -> bool { *self > *o }
    fn ge(&self, o: &f32) -> bool { *self >= *o }
}
