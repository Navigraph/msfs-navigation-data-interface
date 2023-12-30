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
  DownloadNavdata = "DownloadNavdata",
  SetDownloadOptions = "SetDownloadOptions",
  SetActiveDatabase = "SetActiveDatabase",
  ExecuteSQLQuery = "ExecuteSQLQuery",
  GetAirport = "GetAirport",
  GetAirportsInRange = "GetAirportsInRange",
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
