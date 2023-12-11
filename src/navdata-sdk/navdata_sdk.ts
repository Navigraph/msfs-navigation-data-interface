interface CommBusMessage {
  id: string
  resolve: (value?: any) => void
  reject: (reason?: string) => void
}

export enum NavigraphEventType {
  Heartbeat,
  DownloadStatus,
}

enum NavigraphFunction {
  DownloadNavdata,
  SetDownloadOptions,
  SetActiveDatabase,
  ExecuteSQLQuery,
}

interface FunctionReturn {
  status: string
  data: unknown
}

interface Callback {
  event: NavigraphEventType
  callback: (data: unknown) => void
}

interface RawNavigraphEvent {
  event: string
  args: any
}

export class NavigraphNavdataSDK {
  private readonly listener: CommBusListener
  private queue: CommBusMessage[] = []
  private eventListeners: Callback[] = []
  private onReadyCallback: () => void;

  private isInitialized = false

  constructor() {
    this.listener = RegisterCommBusListener(() => {
      this.onRegister()
    })
  }

  public async executeSql(sql: string): Promise<any> {
    return await this.callWasmFunction(NavigraphFunction[NavigraphFunction.ExecuteSQLQuery], { sql })
  }

  public async downloadNavdata(url: string, folder: string): Promise<any> {
    return await this.callWasmFunction(NavigraphFunction[NavigraphFunction.DownloadNavdata], { url, folder })
  }

  public async setActiveDatabase(path: string): Promise<any> {
    return await this.callWasmFunction(NavigraphFunction[NavigraphFunction.SetActiveDatabase], { path })
  }

  private async callWasmFunction(name: string, data: any): Promise<any> {
    let id = Utils.generateGUID()

    let args = {
      function: name,
      id,
      data,
    }

    this.listener.callWasm("NAVIGRAPH_CallFunction", JSON.stringify(args))
    console.log(`[Navigraph] Called ${name} with id ${id}`)

    return new Promise((resolve, reject) => {
      this.queue.push({
        id,
        resolve,
        reject,
      })
    })
  }

  private onRegister(): void {
    this.listener.on("NAVIGRAPH_FunctionResult", (jsonArgs: string) => {
      let args = JSON.parse(jsonArgs)
      let id = args.id
      console.log(`[Navigraph] Received return value for ${id}`)

      let message = this.queue.find(m => m.id === id)
      if (message) {
        this.queue.splice(this.queue.indexOf(message), 1)
        let success = args.status === "success"
        let data = args.data
        if (success) {
          message.resolve(data)
        } else {
          message.reject(data)
        }
      }
    })

    this.listener.on("NAVIGRAPH_Event", (jsonArgs: string) => {
      let args: RawNavigraphEvent = JSON.parse(jsonArgs)
      if (args.event === NavigraphEventType[NavigraphEventType.Heartbeat] && !this.isInitialized) {
        this.isInitialized = true
        if (this.onReadyCallback) {
          this.onReadyCallback()
        }
      }

      // Call callbacks
      if (args.event in NavigraphEventType) {
        let event = NavigraphEventType[args.event as keyof typeof NavigraphEventType]
        let callbacks = this.eventListeners.filter(cb => cb.event === event)
        callbacks.forEach(cb => cb.callback(args.data))
      }
    })
  }

  public onEvent(event: NavigraphEventType, callback: (data: unknown) => void): void {
    let cb: Callback = {
      event,
      callback,
    }
    this.eventListeners.push(cb)
  }

  public onReady(callback: () => void): void {
    this.onReadyCallback = callback
  }

  public getIsInitialized(): boolean {
    return this.isInitialized
  }
}
