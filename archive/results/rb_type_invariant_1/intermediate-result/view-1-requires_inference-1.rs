use vstd::prelude::*;

pub fn main() {}

verus! {
    pub open spec fn ex_saturating_sub_spec(a: int, b: int) -> (ret: nat)
    {
        if (a > b) {
            (a - b) as nat
        } else {
            0
        }
    }

    #[verifier::external_fn_specification]
    pub fn ex_saturating_sub(a: usize, b: usize) -> (ret: usize)
        ensures
            ex_saturating_sub_spec(a as int, b as int) == ret as int,
    {
        a.saturating_sub(b)
    }

    pub struct RingBuffer<T: Copy> {
        ring: Vec<T>,
        head: usize,
        tail: usize,
    }

    impl<T: Copy> View for RingBuffer<T> {
        type V = (Seq<T>, nat);

        closed spec fn view(&self) -> Self::V {
            let ring_len = self.ring.len() as nat;
            let content = if self.head <= self.tail {
                self.ring@.subrange(self.head as int, self.tail as int)
            } else {
                self.ring@.subrange(self.head as int, ring_len as int)
                    ++ self.ring@.subrange(0, self.tail as int)
            };
            (content, ring_len)
        }
    }

    pub open spec fn mod_auto_plus(n: int) -> bool
        recommends
            n > 0,
    {
        forall|x: int, y: int|
            {
                let z = (x % n) + (y % n);
                ((0 <= z < n && #[trigger] ((x + y) % n) == z)
                    || (n <= z < n + n && ((x + y) % n) == z - n))
            }
    }

    pub open spec fn mod_auto_minus(n: int) -> bool
        recommends
            n > 0,
    {
        forall|x: int, y: int|
            {
                let z = (x % n) - (y % n);
                ((0 <= z < n && #[trigger] ((x - y) % n) == z)
                    || (-n <= z < 0 && ((x - y) % n) == z + n))
            }
    }

    pub open spec fn mod_auto(n: int) -> bool
        recommends
            n > 0,
    {
        &&& (n % n == 0 && (-n) % n == 0)
        &&& (forall|x: int| #[trigger] ((x % n) % n) == x % n)
        &&& (forall|x: int| 0 <= x < n <==> #[trigger] (x % n) == x)
        &&& mod_auto_plus(n)
        &&& mod_auto_minus(n)
    }

    pub proof fn lemma_mod_auto(n: int)
        requires
            n > 0,
        ensures
            mod_auto(n),
    {
        admit()
    }

    #[verifier::external_body]
    fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
        requires
            i < old(vec).len(),
        ensures
            vec@ ==
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
// VerusErrorType.Other: unclosed delimiterVerusErrorType.Other: this file contains an unclosed delimiter
// {"$message_type":"diagnostic","message":"this file contains an unclosed delimiter","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwozfh4wl","byte_start":48,"byte_end":49,"line_start":5,"line_end":5,"column_start":8,"column_end":9,"is_primary":false,"text":[{"text":"verus! {","highlight_start":8,"highlight_end":9}],"label":"unclosed delimiter","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwozfh4wl","byte_start":2408,"byte_end":2408,"line_start":93,"line_end":93,"column_start":20,"column_end":20,"is_primary":true,"text":[{"text":"            vec@ ==","highlight_start":20,"highlight_end":20}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: this file contains an unclosed delimiter\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwozfh4wl:93:20\n   |\n5  | verus! {\n   |        - unclosed delimiter\n...\n93 |             vec@ ==\n   |                    ^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
//
//
