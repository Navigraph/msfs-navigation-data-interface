import {
  ArraySubject,
  ComponentProps,
  DisplayComponent,
  EventBus,
  FSComponent,
  Subject,
  VNode,
} from "@microsoft/msfs-sdk";
import { NavigraphNavigationDataInterface } from "@navigraph/msfs-navigation-data-interface";
import "./InterfaceSample.css";
import { NavigationDataStatus } from "@navigraph/msfs-navigation-data-interface";
import { AuthPage } from "./Pages/Auth/Auth";
import { Dashboard } from "./Pages/Dashboard/Dashboard";
import { TestPage } from "./Pages/Test/Test";
import { InterfaceNavbar, InterfaceSwitch } from "./Utils";

interface InterfaceSampleProps extends ComponentProps {
  bus: EventBus;
}

export class InterfaceSample extends DisplayComponent<InterfaceSampleProps> {
  private readonly loadingRef = FSComponent.createRef<HTMLDivElement>();
  private readonly authContainerRef = FSComponent.createRef<HTMLDivElement>();

  private readonly mainPageIndex = Subject.create(0);
  private readonly databaseInfo = Subject.create<NavigationDataStatus | null>(null);

  private navigationDataInterface: NavigraphNavigationDataInterface;

  constructor(props: InterfaceSampleProps) {
    super(props);

    this.navigationDataInterface = new NavigraphNavigationDataInterface();
  }

  public render(): VNode {
    return (
      <>
        <div class="loading-container" ref={this.loadingRef}>
          Waiting for navigation data interface to initialize... If building for the first time, this may take a few
          minutes
        </div>
        <div class="auth-container" ref={this.authContainerRef} style={{ display: "none" }}>
          <div class="size-full flex flex-row bg-ng-background-900">
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
                [0, <Dashboard databaseInfo={this.databaseInfo} interface={this.navigationDataInterface} />],
                [1, <TestPage interface={this.navigationDataInterface} />],
                [
                  2,
                  <AuthPage
                    navigationDataInterface={this.navigationDataInterface}
                    setDatabaseInfo={value => this.databaseInfo.set(value)}
                  />,
                ],
              ]}
            />
          </div>
        </div>
      </>
    );
  }

  public onAfterRender(node: VNode): void {
    super.onAfterRender(node);

    // Populate status when ready
    this.navigationDataInterface.onReady(async () => {
      const activePackage = await this.navigationDataInterface.get_navigation_data_install_status();

      this.databaseInfo.set(activePackage);

      // show the auth container
      this.authContainerRef.instance.style.display = "block";
      this.loadingRef.instance.style.display = "none";
    });
  }
}
