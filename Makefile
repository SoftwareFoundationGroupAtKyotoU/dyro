
mir:
	rustc --emit=mir -Z mir-opt-level=0 -o sample/vec-opt-level-0.mir.rs sample/vec.rs
	rustc --emit=mir -Z mir-opt-level=1 -o sample/vec-opt-level-1.mir.rs sample/vec.rs
	rustc --emit=mir -Z mir-opt-level=2 -o sample/vec-opt-level-2.mir.rs sample/vec.rs
	rustc --emit=mir -Z mir-opt-level=3 -o sample/vec-opt-level-3.mir.rs sample/vec.rs
