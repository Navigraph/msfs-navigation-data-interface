interface CommBusMessage {
    id: string;
    resolve: (value?: any) => void;
    reject: (reason?: any) => void;
}

enum NavigraphEventType {
    Heartbeat
}

interface RawNavigraphEvent {
    event: string;
    args: any;
}

export class NavigraphNavdataSDK {
    private readonly listener: CommBusListener;
    private queue: CommBusMessage[] = [];

    private isInitialized = false;

    constructor() {
        this.listener = RegisterCommBusListener(() => {
            console.info("CommBus listener registered")
            this.onRegsiter();
          });
    }

    public async callWasm(name: string, args: any): Promise<any> {
        let id = Utils.generateGUID();

        args.id = id;
        this.listener.callWasm(name, JSON.stringify(args));
        console.log(`[Navigraph] Called ${name} with id ${id}`);
        
        return new Promise((resolve, reject) => {
            this.queue.push({
                id,
                resolve,
                reject
            });
        });
    }
    
    public onRegsiter(): void {
        this.listener.on("NAVIGRAPH_FunctionResult", (jsonArgs: string) => {
            let args = JSON.parse(jsonArgs);
            let id = args.id;
            delete args.id;
            console.log(`[Navigraph] Received return value for ${id}`);

            let message = this.queue.find(m => m.id === id);
            if (message) {
                this.queue.splice(this.queue.indexOf(message), 1);
                let success = args.status === "success";
                if (success) {
                    message.resolve(args);
                } else {
                    message.reject(args);
                }
            }
        });

        this.listener.on("NAVIGRAPH_Event", (jsonArgs: string) => {
            console.log(jsonArgs);
            let args: RawNavigraphEvent = JSON.parse(jsonArgs);
            if (args.event === NavigraphEventType[NavigraphEventType.Heartbeat] && !this.isInitialized) {
              this.isInitialized = true
              console.log("WASM initialized")
            }
          })
    }

    public getIsInitialized(): boolean {
        return this.isInitialized;
    }
}