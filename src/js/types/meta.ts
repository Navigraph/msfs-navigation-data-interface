export enum InstallStatus {
  Bundled = "Bundled",
  Manual = "Manual",
  None = "None",
}

export interface NavigationDataStatus {
  status: InstallStatus
  installedFormat: string | null
  installedCycle: string | null
  lastestCycle: string | null
}
