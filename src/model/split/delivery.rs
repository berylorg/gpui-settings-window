/// Result of delivering one terminal split page result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPageSplitDelivery {
    /// A coherent page became resident.
    Ready,
    /// The exact range became unavailable.
    Failed,
    /// The exact request was cancelled.
    Cancelled,
    /// The previously issued result no longer belongs to mounted work and was discarded.
    Obsolete,
}

/// Contract failure that leaves current split state and work unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SettingsPageSplitDeliveryError {
    /// The result repeats a different owning page for a current request identity.
    MismatchedPage,
    /// The result repeats a different stable source identity.
    MismatchedSourceIdentity,
    /// The result repeats a different source generation.
    MismatchedGeneration,
    /// The result repeats a different source revision.
    MismatchedRevision,
    /// No current request owns the result's otherwise-current, never-issued request identity.
    MismatchedRequestId,
    /// The result repeats a different exact logical range.
    MismatchedRange,
    /// The result changes the exact focus probe attached to the request.
    MismatchedFocusProbe,
    /// The result repeats a different logical source extent.
    MismatchedLogicalItemCount,
    /// A ready result does not contain exactly one item per requested position.
    IncompleteRange,
    /// Ready items do not repeat the requested contiguous logical positions.
    MalformedLogicalPositions,
    /// A ready item has no stable identity.
    EmptyItemId,
    /// A stable item identity occurs more than once across coherent resident pages.
    DuplicateItemId,
    /// A ready result exceeds the source's hard item limit.
    TooManyItems,
    /// A ready result exceeds the source's hard decoded UTF-8 byte limit.
    TooManyDecodedBytes,
    /// A failed result exceeds the bounded unavailable-message limit.
    FailureMessageTooLarge,
    /// A probed request did not return its required focus resolution.
    MissingFocusResolution,
    /// An unprobed request unexpectedly returned focus resolution metadata.
    UnexpectedFocusResolution,
    /// Focus resolution contradicts the source extent or a coherent resident identity.
    InvalidFocusResolution,
}

impl std::fmt::Display for SettingsPageSplitDeliveryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "split page result rejected: {self:?}")
    }
}

impl std::error::Error for SettingsPageSplitDeliveryError {}
