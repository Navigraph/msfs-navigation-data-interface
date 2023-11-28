# Overview

This document outlines the events used for communication with the WASM module. All data exchanged via these events is in the form of JSON strings. Remember to correctly stringify and parse this data for proper communication!

## Listener Events

Listener events are designed for monitoring and response. **Do not call these events**. Instead, set up listeners to handle them as they get called.

### NAVIGRAPH_Heartbeat

- **Type**: Listener
- **Description**: Triggered every 5 seconds to indicate the WASM module's operational status. Monitoring the first heartbeat is important for verifying module initialization and activity.
- **Data**: None

### NAVIGRAPH_DownloadFailed

- **Type**: Listener
- **Description**: Triggered on a failure in the navdata package download process.
- **Data**: JSON string with an "error" key detailing the failure.
  - **Example**:
    ```json
    {
      "error": "Request timed out"
    }
    ```

### NAVIGRAPH_UnzippedFilesRemaining

- **Type**: Listener
- **Description**: Triggered during navdata package unzipping, useful for displaying download/extraction progress.
- **Data**: JSON string with "total" (total files in archive) and "unzipped" (number of files already unzipped) keys.
  - **Example**:
    ```json
    {
      "total": 100,
      "unzipped": 50
    }
    ```

### NAVIGRAPH_NavdataDownloaded

- **Type**: Listener
- **Description**: Triggered on the completion of navdata package download and extraction.
- **Data**: None

## Callable Events

Callable events are to be actively invoked to interact with the WASM module.

### NAVIGRAPH_DownloadNavdata

- **Type**: Callable
- **Description**: Triggers the download of a navdata package. **Note: there will be a temporary freeze and drop in frames (this can be mitigated by setting download options) due to the downloading and unzipping process. Once it's complete, performance returns to normal**
- **Data**: JSON string with "url" (package URL) and "folder" (target extraction directory under `work/navdata/`) keys.
  - **Example**:
    ```json
    {
      "url": "totallyvalidpackageurl",
      "folder": "avionics"
    }
    ```

### NAVIGRAPH_SetDownloadOptions

- **Type**: Callable
- **Description**: Configures download options, specifically the unzipping batch size to avoid simulation freezing.
- **Data**: JSON string with "batchSize" key (number of files to unzip per frame).
  - **Example**:
    ```json
    {
      "batchSize": 10
    }
    ```

### NAVIGRAPH_DeleteAllNavdata

- **Type**: Callable
- **Description**: Erases all downloaded navdata packages.
- **Data**: None
