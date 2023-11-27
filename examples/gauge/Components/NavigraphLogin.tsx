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

  constructor(props: NavigraphLoginProps) {
    super(props)

    this.commBusListener = RegisterViewListener("JS_LISTENER_COMM_BUS", () => {
      console.info("JS_LISTENER_COMM_BUS registered")
    })

    this.commBusListener.on("NAVIGRAPH_NavdataDownloaded", () => {
      console.info("WASM downloaded navdata")
      this.navdataTextRef.instance.textContent = "Navdata downloaded!"
      this.navdataTextRef.instance.style.color = "white"
    })

    this.commBusListener.on("NAVIGRAPH_UnzippedFilesRemaining", (jsonArgs: string) => {
      const args = JSON.parse(jsonArgs)
      console.info("WASM unzipping files", args)
      const percent = Math.round((args.unzipped / args.total) * 100)
      this.navdataTextRef.instance.textContent = `Unzipping files... ${percent}% complete`
      this.navdataTextRef.instance.style.color = "white"
    })

    this.commBusListener.on("NAVIGRAPH_DownloadFailed", (jsonArgs: string) => {
      const args = JSON.parse(jsonArgs)
      console.error("WASM download failed", args)
      this.navdataTextRef.instance.textContent = `Download failed: ${args.error}`
      // set style to red
      this.navdataTextRef.instance.style.color = "red"
    })
  }

  public render(): VNode {
    return (
      <div class="auth-container">
        <div class="horizontal">
          <div class="vertical">
            <div ref={this.textRef} />
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

    this.loginButtonRef.instance.addEventListener("click", () => this.handleClick().catch(e => console.error(e)))
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
        this.textRef.instance.textContent = "Not signed in"
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
      }, this.cancelSource.token).catch(e => console.error("Failed to sign in!", e))
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
        this.commBusListener.call(
          "COMM_BUS_WASM_CALLBACK",
          "NAVIGRAPH_DownloadNavdata",
          JSON.stringify({
            url,
            folder: pkg.format,
          }),
        )
        this.navdataTextRef.instance.textContent = "Downloading navdata..."
        this.navdataTextRef.instance.style.color = "white"
      })
      .catch(e => console.error(e))
  }
}
