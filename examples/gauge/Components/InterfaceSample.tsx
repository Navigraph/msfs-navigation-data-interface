import {
  ArraySubject,
  ComponentProps,
  DisplayComponent,
  EventBus,
  FSComponent,
  MappedSubject,
  Subject,
  VNode,
} from "@microsoft/msfs-sdk"
import {
  DownloadProgressPhase,
  NavigraphEventType,
  NavigraphNavigationDataInterface,
  PackageInfo,
} from "@navigraph/msfs-navigation-data-interface"
import { CancelToken } from "navigraph/auth"
import { packages } from "../Lib/navigraph"
import { AuthService } from "../Services/AuthService"
import { Dropdown } from "./Dropdown"
import { Input } from "./Input"
import "./InterfaceSample.css"
import { AuthPage } from "./Pages/Auth/Auth"
import { Dashboard } from "./Pages/Dashboard/Dashboard"
import { TestPage } from "./Pages/Test/Test"
import { InterfaceNavbar, InterfaceSwitch } from "./Utils"

interface InterfaceSampleProps extends ComponentProps {
  bus: EventBus
}

export class InterfaceSample extends DisplayComponent<InterfaceSampleProps> {
  private readonly textRef = FSComponent.createRef<HTMLDivElement>()
  private readonly navigationDataTextRef = FSComponent.createRef<HTMLDivElement>()
  private readonly loginButtonRef = FSComponent.createRef<HTMLButtonElement>()
  private readonly qrCodeRef = FSComponent.createRef<HTMLImageElement>()
  private readonly dropdownRef = FSComponent.createRef<Dropdown>()
  private readonly downloadButtonRef = FSComponent.createRef<HTMLButtonElement>()
  private readonly icaoInputRef = FSComponent.createRef<Input>()
  private readonly executeIcaoButtonRef = FSComponent.createRef<HTMLButtonElement>()
  private readonly loadDbRef = FSComponent.createRef<HTMLButtonElement>()
  private readonly sqlInputRef = FSComponent.createRef<Input>()
  private readonly executeSqlButtonRef = FSComponent.createRef<HTMLButtonElement>()
  private readonly outputRef = FSComponent.createRef<HTMLPreElement>()
  private readonly loadingRef = FSComponent.createRef<HTMLDivElement>()
  private readonly authContainerRef = FSComponent.createRef<HTMLDivElement>()

  private readonly activeDatabase = Subject.create<PackageInfo | null>(null)
  private readonly databases = Subject.create<PackageInfo[]>([])
  private readonly mainPageIndex = Subject.create(0)

  private cancelSource = CancelToken.source()

  private navigationDataInterface: NavigraphNavigationDataInterface

  constructor(props: InterfaceSampleProps) {
    super(props)

    this.navigationDataInterface = new NavigraphNavigationDataInterface()

    this.navigationDataInterface.onEvent(NavigraphEventType.DownloadProgress, data => {
      switch (data.phase) {
        case DownloadProgressPhase.Downloading:
          this.displayMessage("Downloading navigation data...")
          break
        case DownloadProgressPhase.Cleaning:
          if (!data.deleted) return
          this.displayMessage(`Cleaning destination directory. ${data.deleted} files deleted so far`)
          break
        case DownloadProgressPhase.Extracting: {
          // Ensure non-null
          if (!data.unzipped || !data.total_to_unzip) return
          const percent = Math.round((data.unzipped / data.total_to_unzip) * 100)
          this.displayMessage(`Unzipping files... ${percent}% complete`)
          break
        }
      }
    })
  }

  public renderDatabaseInfo(): VNode | void {
    return (
      <>
        <div
          class={MappedSubject.create(([status]) => {
            return status ? "vertical" : "hidden"
          }, this.activeDatabase)}
        >
          <div>{this.activeDatabase.map(s => `Bundled: ${s?.is_bundled}`)}</div>
          <div>
            {this.activeDatabase.map(s => `Installed format: ${s?.cycle.format} revision ${s?.cycle.revision}`)}
          </div>
          <div>{this.activeDatabase.map(s => `Active path: ${s?.path}`)}</div>
          <div>{this.activeDatabase.map(s => `Active cycle: ${s?.cycle.cycle}`)}</div>
          <div>{this.activeDatabase.map(s => `Validity period: ${s?.cycle.validityPeriod}`)}</div>
        </div>
      </>
    )
  }

  public render(): VNode {
    return (
      <>
        <div class="loading-container" ref={this.loadingRef}>
          Waiting for navigation data interface to initialize... If building for the first time, this may take a few
          minutes
        </div>
        <div class="auth-container" ref={this.authContainerRef} style={{ display: "none" }}>
          <div class="size-full flex flex-row divide-y bg-ng-background-900">
            <div class="h-full w-[7rem]">
              <InterfaceNavbar
                tabs={[
                  [0, "Dash"],
                  [1, "Test"],
                  [2, "Auth"],
                ]}
                setActive={pageNumber => this.mainPageIndex.set(pageNumber)}
                active={this.mainPageIndex}
              />
            </div>
            <InterfaceSwitch
              class="bg-ng-background-400"
              active={this.mainPageIndex}
              pages={[
                [0, <Dashboard databases={this.databases} />],
                [1, <TestPage />],
                [2, <AuthPage />],
              ]}
            />
          </div>

          {/* <div class="horizontal">
            <div class="vertical">
              <h4>Step 1 - Sign in</h4>
              <div ref={this.textRef}>Loading</div>
              <div ref={this.loginButtonRef} class="button" />
              <div ref={this.navigationDataTextRef} />
              <img ref={this.qrCodeRef} class="qr-code" />
            </div>
            <div class="vertical">
              <h4>Step 2 - Select Database</h4>
              <Dropdown ref={this.dropdownRef} />
              <div ref={this.downloadButtonRef} class="button">
                Download
              </div>
              {this.renderDatabaseInfo()}
            </div>
          </div>

          <h4 style="text-align: center;">Step 3 - Query the database</h4>
          <div class="horizontal">
            <div class="vertical">
              <Input ref={this.icaoInputRef} value="TNCM" class="text-field" />
              <div class="horizontal-no-pad">
                <div ref={this.executeIcaoButtonRef} class="button">
                  Fetch Airport
                </div>
                <div ref={this.loadDbRef} class="button">
                  Load DB
                </div>
              </div>
              <div style="height:30px;"></div>
              <Input
                ref={this.sqlInputRef}
                textarea
                value="SELECT airport_name FROM tbl_airports WHERE airport_identifier = 'TNCM'"
                class="text-field"
              />
              <div ref={this.executeSqlButtonRef} class="button">
                Execute SQL
              </div>
            </div>
            <div class="overflow-scroll h-[400px]">
              <pre ref={this.outputRef} id="output">
                The output of the query will show up here
              </pre>
            </div>
          </div> */}
        </div>
      </>
    )
  }

  public onAfterRender(node: VNode): void {
    super.onAfterRender(node)

    // Populate status when ready
    this.navigationDataInterface.onReady(async () => {
      this.activeDatabase.set(await this.navigationDataInterface.get_active_package())
      this.navigationDataInterface
        .list_available_packages(true)
        .then(pkgs => {
          this.databases.set(pkgs)
        })
        .catch(err => console.error(`Error setting databases: ${err}`))

      // show the auth container
      this.authContainerRef.instance.style.display = "block"
      this.loadingRef.instance.style.display = "none"
    })

    this.loginButtonRef.instance.addEventListener("click", () => this.handleClick())
    this.downloadButtonRef.instance.addEventListener("click", () => this.handleDownloadClick())

    this.executeIcaoButtonRef.instance.addEventListener("click", () => {
      console.time("query")
      this.navigationDataInterface
        .get_arrivals_at_airport(this.icaoInputRef.instance.value)
        .then(procedures => {
          console.info(procedures)
          this.outputRef.instance.textContent = JSON.stringify(procedures, null, 2)
        })
        .catch(e => console.error(e))
        .finally(() => console.timeEnd("query"))
    })

    this.loadDbRef.instance.addEventListener("click", () => this.handleLoadDbClick())

    this.executeSqlButtonRef.instance.addEventListener("click", () => {
      console.time("query")
      this.navigationDataInterface
        .execute_sql(this.sqlInputRef.instance.value, [])
        .then(result => {
          console.info(result)
          this.outputRef.instance.textContent = JSON.stringify(result, null, 2)
        })
        .catch(e => console.error(e))
        .finally(() => console.timeEnd("query"))
    })

    AuthService.user.sub(user => {
      if (user) {
        this.qrCodeRef.instance.src = ""
        this.qrCodeRef.instance.style.display = "none"
        this.loginButtonRef.instance.textContent = "Log out"
        this.textRef.instance.textContent = `Welcome, ${user.preferred_username}`
        this.displayMessage("")

        this.handleLogin()
      } else {
        this.loginButtonRef.instance.textContent = "Sign in"
        this.textRef.instance.textContent = "Not logged in"
      }
    }, true)
  }

  private async handleClick() {
    try {
      if (AuthService.getUser()) {
        await AuthService.signOut()
      } else {
        this.cancelSource = CancelToken.source() // Reset any previous cancellations
        this.displayMessage("Authenticating.. Scan code (or click it) to sign in")
        await AuthService.signIn(p => {
          if (p) {
            this.qrCodeRef.instance.src = `https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${p.verification_uri_complete}`
            this.qrCodeRef.instance.style.display = "block"
            this.qrCodeRef.instance.onclick = () => {
              OpenBrowser(p.verification_uri_complete)
            }
          }
        }, this.cancelSource.token)
      }
    } catch (err) {
      this.qrCodeRef.instance.style.display = "none"
      if (err instanceof Error) this.displayError(err.message)
      else this.displayError(`Unknown error: ${String(err)}`)
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

  private async handleDownloadClick() {
    try {
      if (!this.navigationDataInterface.getIsInitialized()) throw new Error("Navigation data interface not initialized")

      const format = this.dropdownRef.instance.getNavigationDataFormat()
      if (!format) throw new Error("Unable to fetch package: No navigation data format has been selected")

      // Get default package for client
      const pkg = await packages.getPackage(format)

      // Download navigation data to work dir and set active
      await this.navigationDataInterface.download_navigation_data(pkg.file.url, true)

      // Update navigation data status
      this.activeDatabase.set(await this.navigationDataInterface.get_active_package())

      this.displayMessage("Navigation data downloaded")
    } catch (err) {
      if (err instanceof Error) this.displayError(err.message)
      else this.displayError(`Unknown error: ${String(err)}`)
    }
  }

  private async handleLoadDbClick() {
    const data_packages = await this.navigationDataInterface.list_available_packages(true, false)

    this.outputRef.instance.textContent = JSON.stringify(data_packages, null, 2)

    await this.navigationDataInterface.set_active_package(data_packages[0].uuid)
  }

  private displayMessage(message: string) {
    this.navigationDataTextRef.instance.textContent = message
    this.navigationDataTextRef.instance.style.color = "white"
  }

  private displayError(error: string) {
    this.navigationDataTextRef.instance.textContent = error
    this.navigationDataTextRef.instance.style.color = "red"
  }
}
