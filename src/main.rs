#![feature(rustc_private)]
#![feature(box_patterns)]
#![feature(pattern)]

use clap::Parser;

extern crate rustc_apfloat;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

struct DyroCallbacks;

impl rustc_driver::Callbacks for DyroCallbacks {
    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> rustc_driver::Compilation {
        queries.global_ctxt().unwrap().enter(|tcx| {
            for def_id in tcx.mir_keys(()) {
                let mir = tcx.optimized_mir(def_id.to_def_id());

                println!("{:?}: {:#?}", def_id, mir);
            }
        });

        rustc_driver::Compilation::Stop
    }
}

#[derive(clap::Parser, Debug)]
struct DyroArgs {
    rustc_args: Vec<String>,
}

fn main() {
    let args = DyroArgs::parse();
    let rustc_args = {
        let mut rustc_args = vec!["dyro".to_string()];
        rustc_args.append(&mut args.rustc_args.clone());
        rustc_args
    };
    let mut callbacks = DyroCallbacks;
    rustc_driver::RunCompiler::new(&rustc_args, &mut callbacks)
        .run()
        .unwrap();
}
