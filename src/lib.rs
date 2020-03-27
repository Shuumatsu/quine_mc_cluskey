use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::iter::{once, FromIterator, Iterator};

fn calc_ones(mut num: usize) -> usize {
    let mut c = 0;
    while num > 0 {
        c += num & 1;
        num >>= 1;
    }
    c
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AssociatedTerm {
    // 用 binary 记录当前项的二进制形式，用以判断两个项是否相邻
    pub binary: usize,
    // 每次合并相邻项的时候相当于忽略了一个变量，所以我们用 ignored 去记录我们已经忽略了哪些变量
    // 手动计算 Quine–McCluskey 算法的时候通常以在某一位打一个 X 的形式来表示该位对应的变量被忽略
    //     如果要以同样的形式去实现那么我们需要一个 vector 来存储这个“伪二进制”。
    // 在这里的实现为了利用位运算的便利性，用一个额外的二进制量去标记。并且基本类型也更适合 hash
    pub ignored: usize,
    // 用 Petrick's method 从 prime implicant chart 中确定 essential prime implicants 的过程中需要知道当前项蕴含了哪些最小项
    pub min_terms: HashSet<usize>,
}

// 由同样的最小项合成的项应有同样的 hash
impl Hash for AssociatedTerm {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.binary.hash(state);
        self.ignored.hash(state);
    }
}

pub fn pretty_print_term(term: &AssociatedTerm, variables: &Vec<String>) -> String {
    let AssociatedTerm {
        binary, ignored, ..
    } = term;
    let mut factors: Vec<String> = vec![];

    let mut mask = 1_usize << variables.len() - 1;
    for i in 0..variables.len() {
        if ignored & mask == 0 {
            if binary & mask == mask {
                factors.push(format!("{}'", variables[i]));
            } else {
                factors.push(format!("{}", variables[i]));
            }
        }
        mask >>= 1;
    }

    format!("({})", factors.join(" * "))
}

pub fn pretty_print(terms: &HashSet<AssociatedTerm>, variables: &Vec<String>) -> String {
    let mut summands: Vec<String> = vec![];
    for term in terms.iter() {
        summands.push(pretty_print_term(term, variables));
    }
    summands.join(" + ")
}

// Quine–McCluskey 算法的核心思想类似于卡诺图——通过合并相邻项的方式来最小化函数式
// 进行多轮合并操作，每轮操作都将某些已经忽略了相同变量的项合并为一个项
//     每轮中无法合并的项是一个 prime implicant
// 最后再使用 Petrick's method 从 prime implicant chart 中确定 essential prime implicants (没看懂这个算法
pub fn minimize(min_terms: Vec<usize>, variables_c: usize) -> HashSet<AssociatedTerm> {
    let mut groups: Vec<Vec<AssociatedTerm>> = vec![vec![]; variables_c + 1];
    // 根据二进制中 1 的数量进行分类
    for term in min_terms.into_iter() {
        groups[calc_ones(term)].push(AssociatedTerm {
            binary: term,
            ignored: 0,
            min_terms: HashSet::from_iter(once(term)),
        });
    }
    let primes = merge_groups(groups);
    // todo: Petrick's method
    primes
}

fn merge_groups(mut groups: Vec<Vec<AssociatedTerm>>) -> HashSet<AssociatedTerm> {
    // 因为可能出现相同项以不同顺序合并的情况，所以用 set 来剔除相同的项
    let mut primes: HashSet<AssociatedTerm> = HashSet::new();

    while groups.len() > 0 {
        println!("");
        for (oc, group) in groups.iter().enumerate() {
            println!("oc: {}, groups: {:?}", oc, group);
        }
        let mut new_groups: Vec<Vec<AssociatedTerm>> = vec![];
        let mut redundants = HashSet::new();
        for i in 0..groups.len() - 1 {
            let mut new_associated_terms: Vec<AssociatedTerm> = vec![];
            for a_term in groups[i].iter() {
                for b_term in groups[i + 1].iter() {
                    let AssociatedTerm {
                        binary: a_binary,
                        ignored: a_ignored,
                        min_terms: a_min_terms,
                    } = a_term;
                    let AssociatedTerm {
                        binary: b_binary,
                        ignored: b_ignored,
                        min_terms: b_min_terms,
                    } = b_term;

                    // diff 中所有a和b不同的位为1其它位均为0
                    let mut diff = a_binary ^ b_binary;
                    // 但是其中有部分不同的位其实已经被忽略不应该被计算
                    diff = diff - (diff & a_ignored);

                    // 可以验证只有忽略的变量相同的项才有可能合并
                    if a_ignored != b_ignored || calc_ones(diff) > 1 {
                        continue;
                    }

                    redundants.insert(a_term.clone());
                    redundants.insert(b_term.clone());
                    new_associated_terms.push(AssociatedTerm {
                        // 这样可保证如果是由相同的最小项以不同的顺序合并出的项有相同的二进制形式，并与或运算均可
                        binary: a_binary & b_binary,
                        ignored: a_ignored | diff,
                        min_terms: HashSet::from_iter(
                            a_min_terms.iter().chain(b_min_terms.iter()).cloned(),
                        ),
                    });
                }
            }

            new_groups.push(new_associated_terms);
        }
        // 记录在本轮中无法合并的项
        for group in groups.iter() {
            for term in group {
                if !redundants.contains(term) {
                    primes.insert(term.clone());
                }
            }
        }
        println!(
            "\nredundants: {:?} ",
            redundants
                .iter()
                .map(|AssociatedTerm { min_terms, .. }| format!("{:?}", min_terms))
                .collect::<Vec<_>>()
        );
        println!(
            "\nprimes: {:?}",
            primes
                .iter()
                .map(|AssociatedTerm { min_terms, .. }| format!("{:?}", min_terms))
                .collect::<Vec<_>>()
        );
        groups = new_groups;
    }

    primes
}
