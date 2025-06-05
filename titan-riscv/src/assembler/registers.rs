use num_derive::{FromPrimitive, ToPrimitive};
use std::fmt::{Display, Formatter};
/*
0	-	x0	zero	hardwired zero	-
1	-	x1	ra	return address	-R
2	-	x2	sp	stack pointer	-E
3	-	x3	gp	global pointer	-
4	-	x4	tp	thread pointer	-
5	-	x5	t0	temporary register 0	-R
6	-	x6	t1	temporary register 1	-R
7	-	x7	t2	temporary register 2	-R
8	0	x8	s0 / fp	saved register 0 / frame pointer	-E
9	1	x9	s1	saved register 1	-E
10	2	x10	a0	function argument 0 / return value 0	-R
11	3	x11	a1	function argument 1 / return value 1	-R
12	4	x12	a2	function argument 2	-R
13	5	x13	a3	function argument 3	-R
14	6	x14	a4	function argument 4	-R
15	7	x15	a5	function argument 5	-R
16	-	x16	a6	function argument 6	-R
17	-	x17	a7	function argument 7	-R
18	-	x18	s2	saved register 2	-E
19	-	x19	s3	saved register 3	-E
20	-	x20	s4	saved register 4	-E
21	-	x21	s5	saved register 5	-E
22	-	x22	s6	saved register 6	-E
23	-	x23	s7	saved register 7	-E
24	-	x24	s8	saved register 8	-E
25	-	x25	s9	saved register 9	-E
26	-	x26	s10	saved register 10	-E
27	-	x27	s11	saved register 11	-E
28	-	x28	t3	temporary register 3	-R
29	-	x29	t4	temporary register 4	-R
30	-	x30	t5	temporary register 5	-R
31	-	x31	t6	temporary register 6	-R
 */

#[derive(Debug, Copy, Clone, PartialEq, Eq, ToPrimitive, FromPrimitive)]
pub enum RegisterSlot {
    Zero = 0,
    ReturnAddress = 1,
    StackPointer = 2,
    GlobalPointer = 3,
    ThreadPointer = 4,
    Temporary0 = 5,
    Temporary1 = 6,
    Temporary2 = 7,
    Saved0 = 8,
    Saved1 = 9,
    Parameter0 = 10,
    Parameter1 = 11,
    Parameter2 = 12,
    Parameter3 = 13,
    Parameter4 = 14,
    Parameter5 = 15,
    Parameter6 = 16,
    Parameter7 = 17,
    Saved2 = 18,
    Saved3 = 19,
    Saved4 = 20,
    Saved5 = 21,
    Saved6 = 22,
    Saved7 = 23,
    Saved8 = 24,
    Saved9 = 25,
    Saved10 = 26,
    Saved11 = 27,
    Temporary3 = 28,
    Temporary4 = 29,
    Temporary5 = 30,
    Temporary6 = 31,
}

impl RegisterSlot {
    pub fn from_string(input: &str) -> Option<RegisterSlot> {
        Some(match input {
            "zero" => RegisterSlot::Zero,
            "ra" => RegisterSlot::ReturnAddress,
            "sp" => RegisterSlot::StackPointer,
            "gp" => RegisterSlot::GlobalPointer,
            "tp" => RegisterSlot::ThreadPointer,
            "t0" => RegisterSlot::Temporary0,
            "t1" => RegisterSlot::Temporary1,
            "t2" => RegisterSlot::Temporary2,
            "s0" => RegisterSlot::Saved0,
            "fp" => RegisterSlot::Saved0, // ALIAS
            "s1" => RegisterSlot::Saved1,
            "a0" => RegisterSlot::Parameter0,
            "a1" => RegisterSlot::Parameter1,
            "a2" => RegisterSlot::Parameter2,
            "a3" => RegisterSlot::Parameter3,
            "a4" => RegisterSlot::Parameter4,
            "a5" => RegisterSlot::Parameter5,
            "a6" => RegisterSlot::Parameter6,
            "a7" => RegisterSlot::Parameter7,
            "s2" => RegisterSlot::Saved2,
            "s3" => RegisterSlot::Saved3,
            "s4" => RegisterSlot::Saved4,
            "s5" => RegisterSlot::Saved5,
            "s6" => RegisterSlot::Saved6,
            "s7" => RegisterSlot::Saved7,
            "s8" => RegisterSlot::Saved8,
            "s9" => RegisterSlot::Saved9,
            "s10" => RegisterSlot::Saved10,
            "s11" => RegisterSlot::Saved11,
            "t3" => RegisterSlot::Temporary3,
            "t4" => RegisterSlot::Temporary4,
            "t5" => RegisterSlot::Temporary5,
            "t6" => RegisterSlot::Temporary6,

            _ => return None,
        })
    }

    pub fn as_string(&self) -> &str {
        match self {
            RegisterSlot::Zero => "zero",
            RegisterSlot::ReturnAddress => "ra",
            RegisterSlot::StackPointer => "sp",
            RegisterSlot::GlobalPointer => "gp",
            RegisterSlot::ThreadPointer => "tp",
            RegisterSlot::Temporary0 => "t0",
            RegisterSlot::Temporary1 => "t1",
            RegisterSlot::Temporary2 => "t2",
            RegisterSlot::Saved0 => "s0", // ALIAS fp
            RegisterSlot::Saved1 => "s1",
            RegisterSlot::Parameter0 => "a0",
            RegisterSlot::Parameter1 => "a1",
            RegisterSlot::Parameter2 => "a2",
            RegisterSlot::Parameter3 => "a3",
            RegisterSlot::Parameter4 => "a4",
            RegisterSlot::Parameter5 => "a5",
            RegisterSlot::Parameter6 => "a6",
            RegisterSlot::Parameter7 => "a7",
            RegisterSlot::Saved2 => "s2",
            RegisterSlot::Saved3 => "s3",
            RegisterSlot::Saved4 => "s4",
            RegisterSlot::Saved5 => "s5",
            RegisterSlot::Saved6 => "s6",
            RegisterSlot::Saved7 => "s7",
            RegisterSlot::Saved8 => "s8",
            RegisterSlot::Saved9 => "s9",
            RegisterSlot::Saved10 => "s10",
            RegisterSlot::Saved11 => "s11",
            RegisterSlot::Temporary3 => "t3",
            RegisterSlot::Temporary4 => "t4",
            RegisterSlot::Temporary5 => "t5",
            RegisterSlot::Temporary6 => "t6",
        }
    }
}

impl Display for RegisterSlot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.as_string())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ToPrimitive, FromPrimitive)]
pub enum CompressedRegisterSlot {
    Saved0 = 0,
    Saved1 = 1,
    Parameter0 = 2,
    Parameter1 = 3,
    Parameter2 = 4,
    Parameter3 = 5,
    Parameter4 = 6,
    Parameter5 = 7,
}

impl TryFrom<RegisterSlot> for CompressedRegisterSlot {
    type Error = ();

    fn try_from(value: RegisterSlot) -> Result<Self, Self::Error> {
        Ok(match value {
            RegisterSlot::Saved0 => CompressedRegisterSlot::Saved0,
            RegisterSlot::Saved1 => CompressedRegisterSlot::Saved1,
            RegisterSlot::Parameter0 => CompressedRegisterSlot::Parameter0,
            RegisterSlot::Parameter1 => CompressedRegisterSlot::Parameter1,
            RegisterSlot::Parameter2 => CompressedRegisterSlot::Parameter2,
            RegisterSlot::Parameter3 => CompressedRegisterSlot::Parameter3,
            RegisterSlot::Parameter4 => CompressedRegisterSlot::Parameter4,
            RegisterSlot::Parameter5 => CompressedRegisterSlot::Parameter5,
            _ => return Err(()),
        })
    }
}

impl From<CompressedRegisterSlot> for RegisterSlot {
    fn from(value: CompressedRegisterSlot) -> Self {
        match value {
            CompressedRegisterSlot::Saved0 => RegisterSlot::Saved0,
            CompressedRegisterSlot::Saved1 => RegisterSlot::Saved1,
            CompressedRegisterSlot::Parameter0 => RegisterSlot::Parameter0,
            CompressedRegisterSlot::Parameter1 => RegisterSlot::Parameter1,
            CompressedRegisterSlot::Parameter2 => RegisterSlot::Parameter2,
            CompressedRegisterSlot::Parameter3 => RegisterSlot::Parameter3,
            CompressedRegisterSlot::Parameter4 => RegisterSlot::Parameter4,
            CompressedRegisterSlot::Parameter5 => RegisterSlot::Parameter5,
        }
    }
}
