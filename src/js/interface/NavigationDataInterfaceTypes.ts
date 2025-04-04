export * from "../types/meta"

export interface CommBusMessage {
  id: string
  resolve: (value?: unknown) => void
  reject: (reason: Error) => void
}

export enum NavigraphEventType {
  Heartbeat = "Heartbeat",
  DownloadProgress = "DownloadProgress",
}

export enum DownloadProgressPhase {
  Downloading = "Downloading",
  Cleaning = "Cleaning",
  Extracting = "Extracting",
}

export interface DownloadProgressData {
  phase: DownloadProgressPhase
  deleted: number | null
  total_to_unzip: number | null
  unzipped: number | null
}

export enum NavigraphFunction {
  DownloadNavigationData = "DownloadNavigationData",
  SetDownloadOptions = "SetDownloadOptions",
  GetNavigationDataInstallStatus = "GetNavigationDataInstallStatus",
  ExecuteSQLQuery = "ExecuteSQLQuery",
  GetDatabaseInfo = "GetDatabaseInfo",
  GetAirport = "GetAirport",
  GetWaypoints = "GetWaypoints",
  GetVhfNavaids = "GetVhfNavaids",
  GetNdbNavaids = "GetNdbNavaids",
  GetAirways = "GetAirways",
  GetAirwaysAtFix = "GetAirwaysAtFix",
  GetAirportsInRange = "GetAirportsInRange",
  GetWaypointsInRange = "GetWaypointsInRange",
  GetVhfNavaidsInRange = "GetVhfNavaidsInRange",
  GetNdbNavaidsInRange = "GetNdbNavaidsInRange",
  GetAirwaysInRange = "GetAirwaysInRange",
  GetControlledAirspacesInRange = "GetControlledAirspacesInRange",
  GetRestrictiveAirspacesInRange = "GetRestrictiveAirspacesInRange",
  GetCommunicationsInRange = "GetCommunicationsInRange",
  GetRunwaysAtAirport = "GetRunwaysAtAirport",
  GetDeparturesAtAirport = "GetDeparturesAtAirport",
  GetArrivalsAtAirport = "GetArrivalsAtAirport",
  GetApproachesAtAirport = "GetApproachesAtAirport",
  GetWaypointsAtAirport = "GetWaypointsAtAirport",
  GetNdbNavaidsAtAirport = "GetNdbNavaidsAtAirport",
  GetGatesAtAirport = "GetGatesAtAirport",
  GetCommunicationsAtAirport = "GetCommunicationsAtAirport",
  GetGlsNavaidsAtAirport = "GetGlsNavaidsAtAirport",
  GetPathPointsAtAirport = "GetPathPointsAtAirport",
}

export enum FunctionResultStatus {
  Error = "Error",
  Success = "Success",
}

export interface FunctionResultArgs {
  id: string
  status: FunctionResultStatus
  data: unknown
}

export interface Callback<T = unknown> {
  event: NavigraphEventType
  callback: (data: T) => void
}

export interface RawNavigraphEvent {
  event: NavigraphEventType
  data: unknown
}
