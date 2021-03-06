// Adapton uses memoization under the covers, which needs an efficient
// mechanism to search for function pointers and compare them for
// equality.
//
// Meanwhile, Rust does not provide Eq and Hash implementations for
// trait Fn.  So, to identify Rust functions as values that we can
// hash and compare, we need to bundle additional static information
// along with the function pointer as use this data as a proxy for the
// function itself.  The idea is that this information uniquely
// identifies the function pointer (i.e., two distinct functions will
// always have two distinct identities).
//
use std::hash::{Hash,Hasher};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Formatter,Result,Debug};
//use std::mem::replace;

#[derive(PartialEq,Eq,Clone,Hash)]
pub struct ProgPt {
  // Symbolic identity, in Rust semantics:
  pub symbol:&'static str, // via stringify!(...)
  // module:Rc<String>, // via module!()

  // Location in local filesystem:
  //pub file:&'static str,   // via file!()
  //pub line:u32,        // via line!()
  //pub column:u32,      // via column!()
}

impl Debug for ProgPt {
  fn fmt(&self, f: &mut Formatter) -> Result { self.symbol.fmt(f) }
}

pub fn my_hash<T>(obj: T) -> u64
  where T: Hash
{
  let mut hasher = DefaultHasher::new();
  obj.hash(&mut hasher);
  hasher.finish()
}

pub fn my_hash_n<T>(obj: T, n:usize) -> u64
  where T: Hash
{
  let mut hasher = DefaultHasher::new();
  for _ in 0..n {
    obj.hash(&mut hasher);
  }
  hasher.finish()
}

#[macro_export]
macro_rules! prog_pt {
  ($symbol:expr) => {{
    ProgPt{
      symbol:$symbol,
      //file:file!(),
      //line:line!(),
      //column:column!(),
    }
  }}
}

#[macro_export]
macro_rules! thunk {
  [ $nm:expr =>> $suspended_body:expr ] => {{
    thunk
      (ArtIdChoice::Nominal($nm),
       prog_pt!(stringify!("anonymous")),
       Rc::new(Box::new(
         move |(),()|{
           $suspended_body
         })),
       (), 
       ()
      )
  }}
  ;
  ( $nm:expr =>> $f:ident :: < $( $ty:ty ),* > , $( $lab:ident : $arg:expr ),* ) => {{
    thunk
      (ArtIdChoice::Nominal($nm),
       prog_pt!(stringify!($f)),
       Rc::new(Box::new(
         |args, _|{
           let ($( $lab ),*, _) = args ;
           $f :: < $( $ty ),* >( $( $lab ),* )
         })),
       ( $( $arg ),*, ()),
       ()
       )
  }}
  ;
  ( $nm:expr =>> $f:path , $( $lab:ident : $arg:expr ),* ) => {{
    thunk
      (ArtIdChoice::Nominal($nm),
       prog_pt!(stringify!($f)),
       Rc::new(Box::new(
         |args, _|{
           let ($( $lab ),*, _) = args ;
           $f ( $( $lab ),* )
         })),
       ( $( $arg ),*, () ),
       ()
       )
  }}
  ;
  ( $f:ident :: < $( $ty:ty ),* > , $( $lab:ident : $arg:expr ),* ) => {{
    thunk
      (ArtIdChoice::Structural,
       prog_pt!(stringify!($f)),
       Rc::new(Box::new(
         |args, _|{
           let ($( $lab ),*, _) = args ;
           $f :: < $( $ty ),* >( $( $lab ),* )
         })),
       ( $( $arg ),*, () ),
       ()
       )
  }}
  ;
  ( $f:path , $( $lab:ident : $arg:expr ),* ) => {{
    thunk
      (ArtIdChoice::Structural,
       prog_pt!(stringify!($f)),
       Rc::new(Box::new(
         |args, _|{
           let ($( $lab ),*, _) = args ;
           $f ( $( $lab ),* )
         })),
       ( $( $arg ),*, () ),
       ()
       )        
  }}
  ;
  ( $nm:expr =>> $f:ident =>> < $( $ty:ty ),* > , $( $lab1:ident : $arg1:expr ),* ;; $( $lab2:ident : $arg2:expr ),* ) => {{
    let t = thunk
      (ArtIdChoice::Nominal($nm),
       prog_pt!(stringify!($f)),
       Rc::new(Box::new(
         |args1, args2|{
           let ($( $lab1 ),*, _) = args1 ;
           let ($( $lab2 ),*, _) = args2 ;
           $f :: < $( $ty ),* > ( $( $lab1 ),* , $( $lab2 ),* )
         })),
       ( $( $arg1 ),*, () ),
       ( $( $arg2 ),*, () ),
       );
    t
  }}
  ;
}

// #[macro_export]
// macro_rules! thunkic {
//   ( $nm:expr =>> $f:ident :: < $( $ty:ty ),* > , $( $lab:ident : $arg:expr ),* ) => {{
//     thunk
//       (ArtIdChoice::Nominal($nm),
//        prog_pt!(stringify!($f)),
//        Rc::new(Box::new(
//          |args, _|{
//            let ($( $lab ),*, _) = args ;
//            $f :: < $( $ty ),* >( $( $lab ),* )
//          })),
//        ( $( $arg ),*, ()),
//        ()
//        )
//   }}
//   ;
//   ( $nm:expr =>> $f:ident , $( $lab:ident : $arg:expr ),* ) => {{
//     thunk
//       (ArtIdChoice::Nominal($nm),
//        prog_pt!(stringify!($f)),
//        Rc::new(Box::new(
//          |args, _|{
//            let ($( $lab ),*, _) = args ;
//            $f ( $( $lab ),* )
//          })),
//        ( $( $arg ),*, () ),
//        ()
//        )
//   }}
//   ;
//   ( $f:ident :: < $( $ty:ty ),* > , $( $lab:ident : $arg:expr ),* ) => {{
//     thunk
//       (ArtIdChoice::Structural,
//        prog_pt!(stringify!($f)),
//        Rc::new(Box::new(
//          |args, _|{
//            let ($( $lab ),*, _) = args ;
//            $f :: < $( $ty ),* >( $( $lab ),* )
//          })),
//        ( $( $arg ),*, () ),
//        ()
//        )
//   }}
//   ;
//   ( $f:path , $( $lab:ident : $arg:expr ),* ) => {{
//     thunk
//       (ArtIdChoice::Structural,
//        prog_pt!(stringify!($f)),
//        Rc::new(Box::new(
//          |args, _|{
//            let ($( $lab ),*, _) = args ;
//            $f ( $( $lab ),* )
//          })),
//        ( $( $arg ),*, () ),
//        ()
//        )        
//   }}
//   ;
// }

#[macro_export]
macro_rules! memo {
  ( $nm:expr =>> $f:ident :: < $( $ty:ty ),* > , $( $lab:ident : $arg:expr ),* ) => {{
    let t = thunk
      (ArtIdChoice::Nominal($nm),
       prog_pt!(stringify!($f)),
       Rc::new(Box::new(
         |args, _|{
           let ($( $lab ),*) = args ;
           $f :: < $( $ty ),* >( $( $lab ),* )
         })),
       ( $( $arg ),*, ),
       ()
       );
    force(&t)
  }}
  ;
  ( $nm:expr =>> $f:path , $( $lab:ident : $arg:expr ),* ) => {{
    let t = thunk
      (ArtIdChoice::Nominal($nm),
       prog_pt!(stringify!($f)),
       Rc::new(Box::new(
         |args, _|{
           let ($( $lab ),*) = args ;
           $f ( $( $lab ),* )
         })),
       ( $( $arg ),* ),
       ()
       );
    force(&t)
  }}
  ;
  ( $f:ident :: < $( $ty:ty ),* > , $( $lab:ident : $arg:expr ),* ) => {{
    let t = thunk
      (ArtIdChoice::Structural,
       prog_pt!(stringify!($f)),
       Rc::new(Box::new(
         |args, _|{
           let ($( $lab ),*) = args ;
           $f :: < $( $ty ),* >( $( $lab ),* )
         })),
       ( $( $arg ),* ),
       ()
       );
    force(&t)
  }}
  ;
  ( $f:path , $( $lab:ident : $arg:expr ),* ) => {{
    let t = thunk
      (ArtIdChoice::Structural,
       prog_pt!(stringify!($f)),
       Rc::new(Box::new(
         |args, _|{
           let ($( $lab ),*, _) = args ;
           $f ( $( $lab ),* )
         })),
       ( $( $arg ),*, () ),
       ()
       );
    force(&t)
  }}
  ;
  ( $nm:expr =>> $f:path , $( $lab1:ident : $arg1:expr ),* ;; $( $lab2:ident : $arg2:expr ),* ) => {{
    let t = thunk
      (ArtIdChoice::Nominal($nm),
       prog_pt!(stringify!($f)),
       Rc::new(Box::new(
         |args1, args2|{
           let ($( $lab1 ),*, _) = args1 ;
           let ($( $lab2 ),*, _) = args2 ;
           $f ( $( $lab1 ),* , $( $lab2 ),* )
         })),
       ( $( $arg1 ),*, () ),
       ( $( $arg2 ),*, () ),
       );
    force(&t)
  }}
  ;
  ( $nm:expr =>> $f:ident =>> < $( $ty:ty ),* > , $( $lab1:ident : $arg1:expr ),* ;; $( $lab2:ident : $arg2:expr ),* ) => {{
    let t = thunk
      (ArtIdChoice::Nominal($nm),
       prog_pt!(stringify!($f)),
       Rc::new(Box::new(
         |args1, args2|{
           let ($( $lab1 ),*, _) = args1 ;
           let ($( $lab2 ),*, _) = args2 ;
           $f :: < $( $ty ),* > ( $( $lab1 ),* , $( $lab2 ),* )
         })),
       ( $( $arg1 ),*, () ),
       ( $( $arg2 ),*, () ),
       );
    force(&t)
  }}
  ;
}

#[macro_export]
macro_rules! eager {
  ( $nm:expr =>> $f:ident :: < $( $ty:ty ),* > , $( $lab:ident : $arg:expr ),* ) => {{
    let t = thunk
      (ArtIdChoice::Nominal($nm),
       prog_pt!(stringify!($f)),
       Rc::new(Box::new(
         |args, _|{
           let ($( $lab ),*) = args ;
           $f :: < $( $ty ),* >( $( $lab ),* )
         })),
       ( $( $arg ),*, ),
       ()
       );
    let res = force(&t) ;
    (t, res)
  }}
  ;
  ( $nm:expr =>> $f:path , $( $lab:ident : $arg:expr ),* ) => {{
    let t = thunk
      (ArtIdChoice::Nominal($nm),
       prog_pt!(stringify!($f)),
       Rc::new(Box::new(
         |args, _|{
           let ($( $lab ),*) = args ;
           $f ( $( $lab ),* )
         })),
       ( $( $arg ),* ),
       ()
       );
    let res = force(&t) ;
    (t, res)
  }}
  ;
  ( $f:ident :: < $( $ty:ty ),* > , $( $lab:ident : $arg:expr ),* ) => {{
    let t = thunk
      (ArtIdChoice::Structural,
       prog_pt!(stringify!($f)),
       Rc::new(Box::new(
         |args, _|{
           let ($( $lab ),*) = args ;
           $f :: < $( $ty ),* >( $( $lab ),* )
         })),
       ( $( $arg ),* ),
       ()
       );
    let res = force(&t) ;
    (t, res)
  }}
  ;
  ( $f:path , $( $lab:ident : $arg:expr ),* ) => {{
    let t = thunk
      (ArtIdChoice::Structural,
       prog_pt!(stringify!($f)),
       Rc::new(Box::new(
         |args, _|{
           let ($( $lab ),*, _) = args ;
           $f ( $( $lab ),* )
         })),
       ( $( $arg ),*, () ),
       ()
       );
    let res = force(&t) ;
    (t, res)
  }}
  ;
  ( $nm:expr =>> $f:ident =>> < $( $ty:ty ),* > , $( $lab1:ident : $arg1:expr ),* ;; $( $lab2:ident : $arg2:expr ),* ) => {{
    let t = thunk
      (ArtIdChoice::Nominal($nm),
       prog_pt!(stringify!($f)),
       Rc::new(Box::new(
         |args1, args2|{
           let ($( $lab1 ),*, _) = args1 ;
           let ($( $lab2 ),*, _) = args2 ;
           $f :: < $( $ty ),* > ( $( $lab1 ),* , $( $lab2 ),* )
         })),
       ( $( $arg1 ),*, () ),
       ( $( $arg2 ),*, () ),
       );
    let res = force(&t) ;
    (t, res)
  }}
  ;
}

// #[macro_export]
// macro_rules! eageric {
//   ( $nm:expr =>> $f:ident :: < $( $ty:ty ),* > , $( $lab:ident : $arg:expr ),* ) => {{
//     let t = thunk
//       (ArtIdChoice::Nominal($nm),
//        prog_pt!(stringify!($f)),
//        Rc::new(Box::new(
//          |args, _|{
//            let ($( $lab ),*) = args ;
//            $f :: < $( $ty ),* >( $( $lab ),* )
//          })),
//        ( $( $arg ),*, ),
//        ()
//        );
//     let res = force(&t) ;
//     (t, res)
//   }}
//   ;
//   ( $nm:expr =>> $f:path , $( $lab:ident : $arg:expr ),* ) => {{
//     let t = thunk
//       (ArtIdChoice::Nominal($nm),
//        prog_pt!(stringify!($f)),
//        Rc::new(Box::new(
//          |args, _|{
//            let ($( $lab ),*) = args ;
//            $f ( $( $lab ),* )
//          })),
//        ( $( $arg ),* ),
//        ()
//        );
//     let res = force(&t) ;
//     (t, res)
//   }}
//   ;
//   ( $f:ident :: < $( $ty:ty ),* > , $( $lab:ident : $arg:expr ),* ) => {{
//     let t = thunk
//       (ArtIdChoice::Structural,
//        prog_pt!(stringify!($f)),
//        Rc::new(Box::new(
//          |args, _|{
//            let ($( $lab ),*) = args ;
//            $f :: < $( $ty ),* >( $( $lab ),* )
//          })),
//        ( $( $arg ),* ),
//        ()
//        );
//     let res = force(&t) ;
//     (t, res)
//   }}
//   ;
//   ( $f:path , $( $lab:ident : $arg:expr ),* ) => {{
//     let t = thunk
//       (ArtIdChoice::Structural,
//        prog_pt!(stringify!($f)),
//        Rc::new(Box::new(
//          |args, _|{
//            let ($( $lab ),*, _) = args ;
//            $f ( $( $lab ),* )
//          })),
//        ( $( $arg ),*, () ),
//        ()
//        );
//     let res = force(&t) ;
//     (t, res)
//   }}
//   ;
//   ( expr , $nm:expr =>> $f:path , $( $lab1:ident : $arg1:expr ),* ;; $( $lab2:ident : $arg2:expr ),* ) => {{
//     let t = thunk
//       (ArtIdChoice::Nominal($nm),
//        prog_pt!(stringify!($f)),
//        Rc::new(Box::new(
//          |args1, args2|{
//            let ($( $lab1 ),*, _) = args1 ;
//            let ($( $lab2 ),*, _) = args2 ;
//            $f ( $( $lab1 ),* , $( $lab2 ),* )
//          })),
//        ( $( $arg1 ),*, () ),
//        ( $( $arg2 ),*, () ),
//        );
//     let res = force(&t) ;
//     (t, res)
//   }}
//   ;
// }

#[macro_export]
macro_rules! cell_call {
  ( $nm:expr =>> $f:ident :: < $( $ty:ty ),* > , $( $lab:ident : $arg:expr ),* ) => {{
    let res = {
      $f :: < $( $ty ),* >( $( $arg ),*, )
    } ;
    let cell = cell($nm, res) ;
    cell
  }}
  ;
  ( $nm:expr =>> $f:ident , $( $lab:ident : $arg:expr ),* ) => {{
    let res = {
      $f ( $( $arg ),*, )
    } ;
    let cell = cell($nm, res) ;
    cell
  }}

}


// https://doc.rust-lang.org/book/macros.html
//
// macro_rules! o_O {
//     (
//         $(
//             $x:expr; [ $( $y:expr ),* ]
//          );*
//     ) => {
//         &[ $($( $x + $y ),*),* ]
//     }
// }
//
// fn main() {
//     let a: &[i32]
//         = o_O!(10; [1, 2, 3];
//                20; [4, 5, 6]);
//
//     assert_eq!(a, [11, 12, 13, 24, 25, 26]);
// }

// TODO: Need to gensym a variable for each argument below:
//
// macro_rules! thunk {
//     ( $f:ident , $st:expr , $( $arg:expr ),* ) => {
//         let fval = Rc::new(Box::new(
//             |st, args|{
//                 let ($( $arg ),*) = args ;
//                 f( st, $( $arg ),* )
//             })) ;
//         ($st).thunk (ArtId::Eager, prog_pt!($f), fval, $( $arg ),* )
//     }}
