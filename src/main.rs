use std::{
    collections::{HashMap, HashSet},
    io::BufRead,
};

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let g = {
        let mut g = CallGraph::default();

        let mut caller = None;

        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();

        let mut buf: [String; 3] = Default::default();
        loop {
            buf.rotate_right(1);
            buf[0].clear();
            if stdin.read_line(&mut buf[0])? == 0 {
                break;
            }
            if buf[0].ends_with('\n') {
                buf[0].pop();
            }

            let line = &buf[0];
            if let Some(line) = line.strip_prefix("define") {
                let mangled = get_mangled_name(line).ok_or("no name in def")?;
                let name = buf[2].strip_prefix("; ").unwrap_or(mangled);
                let func = g.get_or_create_idx(mangled);
                g.funcs[func].name = name.to_string();

                if caller.replace(func).is_some() {
                    Err("nested fns")?;
                }
            }
            if line == "}" {
                caller = None;
            }

            let call_idx = line
                .find(" call ")
                .or_else(|| line.find(" invoke "))
                .unwrap_or(99);
            if !line.starts_with(';') && call_idx < 40 {
                let caller = caller.ok_or("no caller")?;

                if let Some(mangled) = get_mangled_name(line) {
                    let callee = g.get_or_create_idx(mangled);

                    g.funcs[caller].callees.insert(callee);
                    g.funcs[callee].callers.insert(caller);
                }
            }
        }
        g
    };

    // <ide_ssr::resolving::Resolver>::resolve
    let finish = "_RNvMNtCs1lZaWQ1Khlm_7ide_ssr8matchingNtB2_7Matcher18attempt_match_node";

    // invoke <hashbrown::raw::RawTable<(alloc::sync::Arc<hir_ty::interner::InternedWrapper<chalk_ir::LifetimeData<hir_ty::interner::Interner>>>, dashmap::util::SharedValue<()>)>>::resize::<hashbrown::map::make_hasher<alloc::sync::Arc<hir_ty::interner::InternedWrapper<chalk_ir::LifetimeData<hir_ty::interner::Interner>>>, alloc::sync::Arc<hir_ty::interner::InternedWrapper<chalk_ir::LifetimeData<hir_ty::interner::Interner>>>, dashmap::util::SharedValue<()>, core::hash::BuildHasherDefault<rustc_hash::FxHasher>>::{closure#0}>
    let start = "_RNvMs1_NtNtCsbRUwLxoOw4x_4core3ptr8non_nullINtB5_7NonNullINtNtCscxj2CDt6wGu_5alloc4sync8ArcInnerINtNtCshs3FzZynhAO_6hir_ty8interner15InternedWrapperINtCs7agBoAiVZR2_8chalk_ir6TyDataNtB1z_8InternerEEEE6as_ptrCs1lZaWQ1Khlm_7ide_ssr";

    let start = g.get_idx(start);
    let finish = g.get_idx(finish);

    let callers = g.callers(start);
    let mut curr = finish;

    // loop {
    //     println!("{}", g.funcs[curr].name);
    //     curr = match callers.get(&curr) {
    //         Some(it) => *it,
    //         None => break,
    //     };
    // }

    for (callee, caller) in callers {
        println!("{}\n{}\n", g.funcs[caller].name, g.funcs[caller].mangled);
    }

    Ok(())
}

fn get_mangled_name(line: &str) -> Option<&str> {
    let lo = line.find('@')?;
    let line = &line[lo + 1..];
    let hi = line.find('(')?;
    Some(&line[..hi])
}

#[derive(Default)]
struct CallGraph {
    funcs: Vec<Func>,
    mangled_to_idx: HashMap<String, usize>,
}

impl CallGraph {
    fn get_idx(&self, mangled: &str) -> usize {
        self.mangled_to_idx[mangled]
    }

    fn get_or_create_idx(&mut self, mangled: &str) -> usize {
        let fns = &mut self.funcs;
        *self
            .mangled_to_idx
            .entry(mangled.to_string())
            .or_insert_with(|| {
                let idx = fns.len();
                let mut func = Func::default();
                func.mangled = mangled.to_string();
                fns.push(func);
                idx
            })
    }

    fn callers(&self, start: usize) -> HashMap<usize, usize> {
        let mut links = HashMap::new();
        self.go(start, &mut links);
        links
    }
    fn go(&self, idx: usize, links: &mut HashMap<usize, usize>) {
        for &c in &self.funcs[idx].callers {
            if !links.contains_key(&c) {
                links.insert(c, idx);
                self.go(c, links)
            }
        }
    }
}

#[derive(Default)]
struct Func {
    name: String,
    mangled: String,
    callers: HashSet<usize>,
    callees: HashSet<usize>,
}
