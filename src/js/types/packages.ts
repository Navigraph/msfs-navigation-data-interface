export interface CycleInfo {
  cycle: string
  revision: string
  name: string
  format: string
  validityPeriod: string
}

export interface PackageInfo {
  path: string
  uuid: string
  cycle: CycleInfo
}
