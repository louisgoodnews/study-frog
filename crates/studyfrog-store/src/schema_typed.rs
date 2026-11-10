//! Schema step 27: random typed answer sampling per run (STORY-0136).

pub(crate) const V27: &str = "ALTER TABLE run ADD COLUMN typed_sample INTEGER;\n";
