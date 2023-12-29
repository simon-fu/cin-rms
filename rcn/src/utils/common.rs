use std::{marker::PhantomData, fmt};


pub struct EnumNum<TN, TE>(TN, PhantomData<TE>);

impl<TN: Copy, TE: TryFrom<TN>> EnumNum<TN, TE> {
    pub fn new(num: TN) -> Self {
        Self(num, Default::default())
    }

    pub fn as_num(&self) -> TN {
        self.0
    }

    pub fn as_type(&self) -> Option<TE> {
        TE::try_from(self.0).ok()
    }
}

impl<TN: Copy + fmt::Debug, TE: TryFrom<TN> + fmt::Debug> fmt::Debug for EnumNum<TN, TE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        
        let anum = self.as_num();
        let atype = self.as_type();

        match atype {
            Some(v) => write!(f, "{v:?}({anum:?})")?,
            None => write!(f, "Unknown({anum:?})")?,
        }

        // match atype {
        //     Some(v) => f.write_fmt(format_args!("{v:?}({anum:?})"))?,
        //     None => f.write_fmt(format_args!("Unknown({anum:?})"))?,
        // }
        
        Ok(())
    }
}

pub struct EnumHexU16<TE>(u16, PhantomData<TE>);

impl<TE: TryFrom<u16>> EnumHexU16<TE> {
    pub fn new(num: u16) -> Self {
        Self(num, Default::default())
    }

    pub fn as_num(&self) -> u16 {
        self.0
    }

    pub fn as_type(&self) -> Option<TE> {
        TE::try_from(self.0).ok()
    }
}

impl<TE: TryFrom<u16> + fmt::Debug> fmt::Debug for EnumHexU16<TE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        
        let anum = self.as_num();
        let atype = self.as_type();

        match atype {
            Some(v) => write!(f, "{v:?}(0x{anum:04x?})")?,
            None => write!(f, "Unknown({anum:?})")?,
        }
        
        Ok(())
    }
}

// // #[derive(Debug)]
// struct Foo(pub i32);

// impl fmt::Debug for Foo {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let value = self.0;
//         match value {
//             100 => write!(f, "OneHundred({value})")?,
//             _ => write!(f, "Unknown({value})")?,
//         }
//         Ok(())
//     }
// }

// #[test]
// fn test_foo() {
//     println!("{:02X?}", Foo(100));
// }
