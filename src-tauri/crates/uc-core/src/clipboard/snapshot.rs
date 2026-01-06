#[derive(Debug, Clone)]
pub struct ClipboardDecisionSnapshot {
    pub blobs_exist: bool,
}

impl ClipboardDecisionSnapshot {
    /// Constructs a `ClipboardDecisionSnapshot` indicating whether clipboard blobs exist.
    ///
    /// The `blobs_exist` flag records whether clipboard blob data is present and is stored in the returned snapshot.
    ///
    /// # Examples
    ///
    /// ```
    /// let snap = ClipboardDecisionSnapshot::new(true);
    /// assert!(snap.is_usable());
    /// ```
    pub fn new(blobs_exist: bool) -> Self {
        Self { blobs_exist }
    }

    /// Indicates whether the snapshot represents a usable clipboard (i.e., contains blobs).
    ///
    /// Returns `true` if blobs exist, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let s = ClipboardDecisionSnapshot::new(true);
    /// assert!(s.is_usable());
    ///
    /// let s2 = ClipboardDecisionSnapshot::new(false);
    /// assert!(!s2.is_usable());
    /// ```
    pub fn is_usable(&self) -> bool {
        self.blobs_exist
    }
}