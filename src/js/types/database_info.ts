export interface DatabaseInfo {
  /// The AIRAC cycle that this database is.
  ///
  /// e.g. `2313` or `2107`
  airac_cycle: string
  /// The effective date range of this AIRAC cycle.
  effective_from_to: [string, string]
  /// The effective date range of the previous AIRAC cycle
  previous_from_to: [string, string]
}
