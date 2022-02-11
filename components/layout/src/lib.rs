pub mod box_model;
pub mod flow;
pub mod formatting_context;
pub mod layout_box;
pub mod tree_builder;
pub mod text;

#[macro_export]
macro_rules! dump_layout {
    ($node:expr) => {
        $node.dump(0)
            .lines()
            .for_each(|line| log::debug!("{}", line));
    };
}

#[cfg(test)]
pub mod utils;
