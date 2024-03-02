use fathom_syzygy::Fathom;

/// Check if the [`Fathom`] struct is drop safe.
///
/// In version 0.1.0 of the `fathom-syzygy` crate, the [`Fathom`] struct is not drop safe. This
/// is illustrated by this test case, resulting in a memory corruption error.
#[test]
fn test_drop_safety() -> Result<(), Box<dyn std::error::Error>> {
    {
        // This doesn't even need a real path, just any non-empty string will do.
        Fathom::new("foobar")?;
    }
    {
        // The second time it can be anything, even an empty string.
        Fathom::new("")?;
    }
    Ok(())
}
