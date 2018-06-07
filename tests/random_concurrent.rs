extern crate disjoint_sets;

#[macro_use]
extern crate quickcheck;

use disjoint_sets::{UnionFind, AUnionFind};
use quickcheck::{Arbitrary, Gen};
use std::sync::Arc;
use std::thread;

// The length of the union-find we'll test on.
const UF_LEN: usize = 100;

// The percentage of commands that should be finds; the rest are unions.
const FIND_PCT: usize = 80;

// The maximum length of each generated script.
const MAX_SCRIPT_LEN: usize = 200;

// The number of threads to start.
const CONCURRENCY: usize = 10;

quickcheck! {
    fn prop_a_union_find_simulates_union_find(multi_script: MultiScript) -> bool {
        let mut tester = Tester::new();
        tester.execute(&multi_script);
        tester.check()
    }
}

// We will run the same operations on an `AUnionFind` and a `UnionFind`,
// and then check that the results are equivalent.
struct Tester {
    concurrent: Arc<AUnionFind>,
    sequential: UnionFind<usize>,
    set_count:  usize,
}

impl Tester {
    // Creates a fresh tester.
    fn new() -> Self {
        Tester {
            concurrent: Arc::new(AUnionFind::new(UF_LEN)),
            sequential: UnionFind::new(UF_LEN),
            set_count:  UF_LEN,
        }
    }

    // Checks that the two union-finds in the tester are equivalent.
    fn check(&self) -> bool {
//        eprintln!("set_count: {}", self.set_count);

        for i in 0 .. UF_LEN {
            for j in 0 .. UF_LEN {
                if self.concurrent.equiv(i, j) != self.sequential.equiv(i, j) {
                    return false;
                }
            }
        }

        true
    }

    // Executes the given multi-script on both union-finds.
    fn execute(&mut self, multi_script: &MultiScript) {
        // First execute the scripts sequentially:
        for script in &multi_script.0 {
            for cmd in &script.0 {
                match *cmd {
                    Cmd::Union(i, j) =>
                        if self.sequential.union(i, j) {
                            self.set_count -= 1;
                        }
                    Cmd::Find(i)     => { self.sequential.find(i); }
                }
            }
        }

        // Next we'll do the concurrent version. We clone the multi-script
        // so we can hand off ownership of a script to each thread.
        let mut handles = Vec::with_capacity(multi_script.0.len());
        for script in multi_script.clone().0 {
            let uf = self.concurrent.clone();
            handles.push(thread::spawn(move || {
                for cmd in script.0 {
                    match cmd {
                        Cmd::Union(i, j) => { uf.union(i, j); }
                        Cmd::Find(i)     => { uf.find(i); }
                    }
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}

// Multiple scripts, one for each thread.
#[derive(Clone, Debug)]
struct MultiScript(Vec<Script>);

// A script is a sequence of commands.
#[derive(Clone, Debug)]
struct Script(Vec<Cmd>);

// A command is either a union or a find.
#[derive(Clone, Debug)]
enum Cmd {
    Union(usize, usize),
    Find(usize),
}

impl Arbitrary for MultiScript {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let mut result = Vec::with_capacity(CONCURRENCY);

        for _ in 0 .. CONCURRENCY {
            result.push(Script::arbitrary(g));
        }

        MultiScript(result)
    }

    fn shrink(&self) -> Box<Iterator<Item=Self>> {
        Box::new(
            self.0.shrink()
                .flat_map(|scripts| scripts.shrink())
                .map(MultiScript)
        )
    }
}

impl Arbitrary for Script {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let len = g.gen_range(0, MAX_SCRIPT_LEN);
        let mut result = Vec::with_capacity(len);

        for _ in 0 .. len {
            result.push(Cmd::arbitrary(g))
        }

        Script(result)
    }

    fn shrink(&self) -> Box<Iterator<Item=Self>> {
        Box::new(self.0.shrink().map(Script))
    }
}

impl Arbitrary for Cmd {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let choice = g.gen_range(1, 101);
        let mut gen_index = || g.gen_range(0, UF_LEN);

        match choice {
            1...FIND_PCT => Cmd::Find(gen_index()),
            _            => Cmd::Union(gen_index(), gen_index()),
        }
    }
}

