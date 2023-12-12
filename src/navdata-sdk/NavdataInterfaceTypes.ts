export interface CommBusMessage {
  id: string
  resolve: (value?: unknown) => void
  reject: (reason: Error) => void
}

export enum NavigraphEventType {
  Heartbeat,
  DownloadProgress,
}

export enum DownloadProgressPhase {
  Downloading, // 0
  Cleaning, // 1
  Extracting, // 2
}

export interface DownloadProgressData {
  phase: DownloadProgressPhase
  deleted: number | null
  total_to_unzip: number | null
  unzipped: number | null
}

export enum NavigraphFunction {
  DownloadNavdata,
  SetDownloadOptions,
  SetActiveDatabase,
  ExecuteSQLQuery,
}

export enum FunctionResultStatus {
  Error, // 0
  Success, // 1
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
  event: string
  data: unknown
}
