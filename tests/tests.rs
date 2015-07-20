#![feature(test)]
#[macro_use]
extern crate adapton ;

mod zipper {
    extern crate test;
    use adapton::adapton_syntax::* ;
    use adapton::adapton_sigs::* ;
    //use adapton::adapton_state::* ;
    use adapton::adapton_fromscratch::* ;
    use adapton::structures::* ;
    
    pub fn zipper_edits<A:Adapton,L:ListT<A,u64>> (st:&mut A, nil:L) {
        let z = ListZipper::<A,u64,L>::empty(&mut st);
        println!("{:?}", z)
    }
    
    #[test]
    pub fn zipper_test () {
        let mut st = AdaptonFromScratch::new();
        let nil = List::nil(st);
        zipper_edits (&mut st, nil)
    }
}


#[cfg(test)]
mod pure_caching {
    extern crate test;
    use self::test::Bencher;
    use std::rc::Rc;   
    use adapton::adapton_syntax::* ;
    use adapton::adapton_sigs::* ;
    use adapton::adapton_state::* ;
    
    pub fn fact<A:Adapton> (st:&mut A, x:u64, _n:() ) -> u64 {
        if x == 0 { 1 } else { x * (memo!(st, fact, x:x-1, _n:())) }
    }
    
    pub fn run_fact (x:u64) -> u64 {
        let mut st = &mut (AdaptonState::new()) ;
        memo!(st, fact, x:x, _n:())
    }

    pub fn run_fact_repeat (x:u64, n:u64) -> u64 {
        let mut st = &mut (AdaptonState::new()) ;
        for _ in 1..(n-1) {
            memo!(st, fact, x:x, _n:());
        }
        memo!(st, fact, x:x, _n:())
    }

    #[test]
    fn it_works() {
        assert_eq!(120 as u64, run_fact(5));
    }
    
    #[bench]
    fn bench_fact_5(b: &mut Bencher) {
        b.iter(|| run_fact(5));
    }

    #[bench]
    fn bench_fact_5_repeat_100(b: &mut Bencher) {
        b.iter(|| run_fact_repeat(5, 100));
    }
}
