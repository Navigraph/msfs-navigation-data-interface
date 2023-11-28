import { ComponentProps, DisplayComponent, EventBus, FSComponent, VNode } from "@microsoft/msfs-sdk"
import { CancelToken, navigraphRequest } from "navigraph/auth"
import { packages } from "../Lib/navigraph"
import { AuthService } from "../Services/AuthService"
import "./NavigraphLogin.css"
import { Dropdown } from "./Dropdown"

interface NavigraphLoginProps extends ComponentProps {
  bus: EventBus
}

export class NavigraphLogin extends DisplayComponent<NavigraphLoginProps> {
  private readonly textRef = FSComponent.createRef<HTMLDivElement>()
  private readonly navdataTextRef = FSComponent.createRef<HTMLDivElement>()
  private readonly loginButtonRef = FSComponent.createRef<HTMLButtonElement>()
  private readonly qrCodeRef = FSComponent.createRef<HTMLImageElement>()
  private readonly dropdownRef = FSComponent.createRef<Dropdown>()
  private readonly downloadButtonRef = FSComponent.createRef<HTMLButtonElement>()

  private cancelSource = CancelToken.source()

  private commBusListener: ViewListener.ViewListener

  private wasmInitialized = false

  constructor(props: NavigraphLoginProps) {
    super(props)

    this.commBusListener = RegisterViewListener("JS_LISTENER_COMM_BUS", () => {
      console.info("JS_LISTENER_COMM_BUS registered")
    })

    this.commBusListener.on("NAVIGRAPH_Heartbeat", () => {
      if (!this.wasmInitialized) {
        this.wasmInitialized = true
        console.log("WASM initialized")
      }
    })

    this.commBusListener.on("NAVIGRAPH_NavdataDownloaded", () => {
      console.info("WASM downloaded navdata")
      this.displayMessage("Navdata downloaded")
    })

    this.commBusListener.on("NAVIGRAPH_UnzippedFilesRemaining", (jsonArgs: string) => {
      const args = JSON.parse(jsonArgs)
      console.info("WASM unzipping files", args)
      const percent = Math.round((args.unzipped / args.total) * 100)
      this.displayMessage(`Unzipping files... ${percent}% complete`)
    })

    this.commBusListener.on("NAVIGRAPH_DownloadFailed", (jsonArgs: string) => {
      const args = JSON.parse(jsonArgs)
      this.displayError(args.error)
    })
  }

  public render(): VNode {
    return (
      <div class="auth-container">
        <div class="horizontal">
          <div class="vertical">
            <div ref={this.textRef}>Loading</div>
            <div ref={this.loginButtonRef} class="button" />
            <div ref={this.navdataTextRef} />
            <img ref={this.qrCodeRef} class="qr-code" />
          </div>
          <div class="vertical">
            <Dropdown ref={this.dropdownRef} />
            <div ref={this.downloadButtonRef} class="button">
              Download
            </div>
          </div>
        </div>
      </div>
    )
  }

  public onBeforeRender(): void {
    super.onBeforeRender()
  }

  public onAfterRender(node: VNode): void {
    super.onAfterRender(node)

    this.loginButtonRef.instance.addEventListener("click", () =>
      this.handleClick().catch(e => this.displayError(e.message)),
    )
    this.downloadButtonRef.instance.addEventListener("click", () => this.handleDownloadClick())

    AuthService.user.sub(user => {
      if (user) {
        this.qrCodeRef.instance.src = ""
        this.qrCodeRef.instance.style.display = "none"
        this.loginButtonRef.instance.textContent = "Log out"
        this.textRef.instance.textContent = `Welcome, ${user.preferred_username}`

        this.handleLogin()
      } else {
        this.loginButtonRef.instance.textContent = "Sign in"
        this.textRef.instance.textContent = "Not logged in"
      }
    }, true)
  }

  private async handleClick() {
    if (AuthService.getUser()) {
      await AuthService.signOut()
    } else {
      this.cancelSource = CancelToken.source() // Reset any previous cancellations
      AuthService.signIn(p => {
        if (p) {
          this.qrCodeRef.instance.src = `https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${p.verification_uri_complete}`
          this.qrCodeRef.instance.style.display = "block"
          console.info(p.verification_uri_complete)
        }
      }, this.cancelSource.token).catch(e => this.displayError(e.message))
    }
  }

  private handleLogin() {
    // Let's display all of our packages
    packages
      .listPackages()
      .then(pkgs => {
        for (const pkg of pkgs) {
          this.dropdownRef.instance.addDropdownItem(pkg.format, pkg.format)
        }
      })
      .catch(e => console.error(e))
  }

  private handleDownloadClick() {
    packages
      .getPackage(this.dropdownRef.instance.getNavdataFormat() as string)
      .then(pkg => {
        const url = pkg.file.url
        // eslint-disable-next-line @typescript-eslint/no-floating-promises
        if (this.wasmInitialized) {
          this.commBusListener.call(
            "COMM_BUS_WASM_CALLBACK",
            "NAVIGRAPH_DownloadNavdata",
            JSON.stringify({
              url,
              folder: pkg.format,
            }),
          )
          this.displayMessage("Downloading navdata...")
        } else {
          this.displayError("WASM not initialized")
        }
      })
      .catch(e => this.displayError(e.message))
  }

  private displayMessage(message: string) {
    this.navdataTextRef.instance.textContent = message
    this.navdataTextRef.instance.style.color = "white"
  }

  private displayError(error: string) {
    this.navdataTextRef.instance.textContent = error
    this.navdataTextRef.instance.style.color = "red"
  }
}
