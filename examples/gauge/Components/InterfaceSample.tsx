import {
  ArraySubject,
  ComponentProps,
  DisplayComponent,
  EventBus,
  FSComponent,
  Subject,
  VNode,
} from "@microsoft/msfs-sdk"
import { NavigraphNavigationDataInterface, PackageInfo } from "@navigraph/msfs-navigation-data-interface"
import "./InterfaceSample.css"
import { AuthPage } from "./Pages/Auth/Auth"
import { Dashboard } from "./Pages/Dashboard/Dashboard"
import { TestPage } from "./Pages/Test/Test"
import { InterfaceNavbar, InterfaceSwitch } from "./Utils"

interface InterfaceSampleProps extends ComponentProps {
  bus: EventBus
}

export class InterfaceSample extends DisplayComponent<InterfaceSampleProps> {
  private readonly loadingRef = FSComponent.createRef<HTMLDivElement>()
  private readonly authContainerRef = FSComponent.createRef<HTMLDivElement>()

  private readonly activeDatabase = Subject.create<PackageInfo | null>(null)
  private readonly databases = ArraySubject.create<PackageInfo>([])
  private readonly mainPageIndex = Subject.create(0)
  private readonly selectedDatabaseIndex = Subject.create(0)
  private readonly selectedDatabase = Subject.create<PackageInfo | null>(null)

  private navigationDataInterface: NavigraphNavigationDataInterface

  constructor(props: InterfaceSampleProps) {
    super(props)

    this.navigationDataInterface = new NavigraphNavigationDataInterface()
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
                [
                  0,
                  <Dashboard
                    activeDatabase={this.activeDatabase}
                    databases={this.databases}
                    selectedDatabase={this.selectedDatabase}
                    selectedDatabaseIndex={this.selectedDatabaseIndex}
                    setSelectedDatabase={database => this.selectedDatabase.set(database)}
                    setSelectedDatabaseIndex={index => this.selectedDatabaseIndex.set(index)}
                    interface={this.navigationDataInterface}
                  />,
                ],
                [1, <TestPage />],
                [
                  2,
                  <AuthPage
                    activeDatabase={this.activeDatabase}
                    setActiveDatabase={database => this.activeDatabase.set(database)}
                    navigationDataInterface={this.navigationDataInterface}
                  />,
                ],
              ]}
            />
          </div>

          {/* 
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
      const pkgs = await this.navigationDataInterface.list_available_packages(true)

      this.databases.set(pkgs)

      const activePackage = await this.navigationDataInterface.get_active_package()

      this.activeDatabase.set(activePackage)
      this.selectedDatabase.set(activePackage)
      if (activePackage !== null) {
        this.selectedDatabaseIndex.set(pkgs.findIndex(pkg => pkg.uuid === activePackage.uuid))
      }

      // show the auth container
      this.authContainerRef.instance.style.display = "block"
      this.loadingRef.instance.style.display = "none"
    })

    // this.executeIcaoButtonRef.instance.addEventListener("click", () => {
    //   console.time("query")
    //   this.navigationDataInterface
    //     .get_arrivals_at_airport(this.icaoInputRef.instance.value)
    //     .then(procedures => {
    //       console.info(procedures)
    //       this.outputRef.instance.textContent = JSON.stringify(procedures, null, 2)
    //     })
    //     .catch(e => console.error(e))
    //     .finally(() => console.timeEnd("query"))
    // })

    // this.loadDbRef.instance.addEventListener("click", () => this.handleLoadDbClick())

    // this.executeSqlButtonRef.instance.addEventListener("click", () => {
    //   console.time("query")
    //   this.navigationDataInterface
    //     .execute_sql(this.sqlInputRef.instance.value, [])
    //     .then(result => {
    //       console.info(result)
    //       this.outputRef.instance.textContent = JSON.stringify(result, null, 2)
    //     })
    //     .catch(e => console.error(e))
    //     .finally(() => console.timeEnd("query"))
    // })
  }
}
