export enum InstallStatus {
  Bundled = "Bundled",
  Manual = "Manual",
  None = "None",
}

export interface NavigationDataStatus {
  status: InstallStatus
  installedFormat: string | null
  installedRevision: string | null
  installedCycle: string | null
  installedPath: string | null
  validityPeriod: string | null
  latestCycle: string | null
}
