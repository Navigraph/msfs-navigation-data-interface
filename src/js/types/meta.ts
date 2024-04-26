export enum InstallStatus {
  Bundled = "Bundled",
  Manual = "Manual",
  None = "None",
}

export interface NavigationDataStatus {
  status: InstallStatus
  installedFormat: string | null
  installedRegion: string | null
  installedCycle: string | null
  validityPeriod: string | null
  lastestCycle: string | null
}
