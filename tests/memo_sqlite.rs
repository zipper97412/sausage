use sausage::MemoizedFsWalker;

mod common;
use common::*;

use anyhow::Result;

use crate::common::TestProcessor;

#[test]
fn test1() -> Result<()> {
    let mut acc = Vec::with_capacity(4096);
    let proc = TestProcessor::new(&mut acc)?;
    let tmpdir = new_tmpdir("test1")?;
    let mut db = new_sqlite_cache(&tmpdir, "cache.db")?;

    let testdir = new_asset_full(&tmpdir, "asset")?;
    {
        let tx = db.transaction()?;
        let mut walker = MemoizedFsWalker::new(proc);
        // WORKSPACE
        let _ = walker.hash_path(&*tx, &testdir)?;
        tx.commit()?;
    }

    // CHECK\
    let result = std::str::from_utf8(&acc)?;
    let expected = r#"
F|testing/a
F|testing/1/b
D|testing/1|1
S|testing/c
D|testing|3
"#;
    assert_eq!(
        result, expected,
        "\nresult: \n{}\nexpected: \n{}",
        result, expected
    );
    Ok(())
}
