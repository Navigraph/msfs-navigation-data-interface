import {
  ComponentProps,
  DisplayComponent,
  FSComponent,
  MappedSubject,
  MappedSubscribable,
  MutableSubscribable,
  Subscribable,
  SubscribableArray,
  VNode,
} from "@microsoft/msfs-sdk"
import { NavigraphNavigationDataInterface, PackageInfo } from "@navigraph/msfs-navigation-data-interface"
import { List } from "../../List"
import { Button, InterfaceNavbarItemV2, InterfaceSwitch } from "../../Utils"

interface DashboardProps extends ComponentProps {
  databases: SubscribableArray<PackageInfo>
  reloadPackageList: MutableSubscribable<boolean>
  selectedDatabase: Subscribable<PackageInfo | null>
  selectedDatabaseIndex: Subscribable<number>
  setSelectedDatabase: (database: PackageInfo) => void
  setSelectedDatabaseIndex: (index: number) => void
  activeDatabase: Subscribable<PackageInfo | null>
  interface: NavigraphNavigationDataInterface
}

export class Dashboard extends DisplayComponent<DashboardProps> {
  private readonly _selectedCallback = this.props.selectedDatabaseIndex.map(val => {
    if (this.props.databases.length !== 0) {
      this.props.setSelectedDatabase(this.props.databases.get(val))
    }
  })
  private readonly showActiveDatabase = MappedSubject.create(
    ([selectedDatabase, activeDatabase]) => selectedDatabase?.uuid === activeDatabase?.uuid,
    this.props.selectedDatabase,
    this.props.activeDatabase,
  )

  private displayItems(data: PackageInfo, index: number): VNode {
    return (
      <InterfaceNavbarItemV2
        class="w-full p-4 flex items-center hover:bg-blue-800"
        activeClass="bg-blue-400"
        content={""}
        active={this.props.selectedDatabaseIndex.map(val => val === index)}
        setActive={() => this.props.setSelectedDatabaseIndex(index)}
      >
        <p class="text-2xl text-inherit">
          {data.cycle.cycle} - {data.cycle.format}
        </p>
      </InterfaceNavbarItemV2>
    )
  }

  private setDatabase() {
    const selectedDatabase = this.props.selectedDatabase.get()

    if (selectedDatabase === null) {
      return
    }

    this.props.interface
      .set_active_package(selectedDatabase.uuid)
      .then(_res => {})
      .catch(err => console.error(err))

    this.props.reloadPackageList.set(true)
  }

  private deleteSelected() {
    const prevSelectedDatabase = this.props.selectedDatabase.get()

    if (prevSelectedDatabase === null || this.props.databases.length <= 1) {
      return
    }

    let pkg

    try {
      if (this.props.selectedDatabaseIndex.get() === 0) {
        pkg = this.props.databases.get(1)
      } else {
        pkg = this.props.databases.get(0)
      }
    } catch {
      return
    }

    if (this.showActiveDatabase.get()) {
      this.props.interface
        .set_active_package(pkg.uuid)
        .then(_res => {})
        .catch(err => console.error(err))
    }

    this.props.interface
      .delete_package(prevSelectedDatabase.uuid)
      .then(_res => {})
      .catch(err => console.error(err))

    this.props.reloadPackageList.set(true)
  }

  render(): VNode {
    return (
      <div class="size-full flex flex-col">
        <p class="ml-2 mb-8 text-4xl">Dashboard</p>
        <div class="flex flex-row flex-grow flex-auto">
          <div class="w-1/3 flex flex-col">
            <p class="ml-2 text-3xl mb-4">Databases</p>
            <div class="mt-2 flex-grow bg-ng-background-500 shadow-inner">
              <div class="flex flex-col space-y-2 overflow-auto">
                <List
                  data={this.props.databases}
                  renderItem={(data, index) => this.displayItems(data as PackageInfo, index)}
                />
              </div>
            </div>
            <div class="flex flex-row">
              <Button onClick={() => this.setDatabase()} class="p-4 bg-blue-400 hover:bg-blue-800 flex-grow w-full">
                <p class="text-2xl">Select Database</p>
              </Button>
              <Button
                onClick={() => this.deleteSelected()}
                class="bg-red-900 hover:bg-red-500 w-[25%] flex items-center justify-center"
              >
                <span class="text-5xl">X</span>
              </Button>
            </div>
          </div>
          <ActiveDatabase selectedDatabase={this.props.selectedDatabase} showActiveDatabase={this.showActiveDatabase} />
        </div>
      </div>
    )
  }
}

interface ActiveDatabaseProps extends ComponentProps {
  selectedDatabase: Subscribable<PackageInfo | null>
  showActiveDatabase: MappedSubscribable<boolean>
}

class ActiveDatabase extends DisplayComponent<ActiveDatabaseProps> {
  private readonly isActive = this.props.showActiveDatabase.map(val => (val ? 0 : 1))

  render(): VNode {
    return (
      <div class="w-2/3 flex flex-col">
        <InterfaceSwitch
          class="ml-2 flex flex-row"
          active={this.isActive}
          noTheming={true}
          intoNoTheming={true}
          pages={[
            [0, <p class="text-3xl mb-4">Active Database</p>],
            [1, <p class="text-3xl mb-4">Selected Database</p>],
          ]}
        />
        <div class="mt-2 flex-grow bg-ng-background-700 overflow-auto">
          <div class="p-4 flex flex-col align-middle items-start flex-start space-y-6 vertical">
            <div class="flex flex-row space-x-2">
              <span class="text-2xl">UUID:</span>
              <span class="text-xl text-gray-400">{this.props.selectedDatabase.map(s => s?.uuid)}</span>
            </div>
            <div class="flex flex-col space-y-2">
              <p class="text-3xl">Bundled</p>
              <p class="text-2xl text-gray-400">{this.props.selectedDatabase.map(s => s?.is_bundled)}</p>
            </div>
            <div class="flex flex-col space-y-2">
              <p class="text-3xl">Installed format</p>
              <p class="text-2xl text-gray-400">
                {this.props.selectedDatabase.map(s => `${s?.cycle.format} revision ${s?.cycle.revision}`)}
              </p>
            </div>
            <div class="flex flex-col space-y-2">
              <p class="text-3xl">Active path</p>
              <p class="text-2xl text-gray-400">{this.props.selectedDatabase.map(s => s?.path)}</p>
            </div>
            <div class="flex flex-col space-y-2">
              <p class="text-3xl">Active cycle</p>
              <p class="text-2xl text-gray-400">{this.props.selectedDatabase.map(s => s?.cycle.cycle)}</p>
            </div>
            <div class="flex flex-col space-y-2">
              <p class="text-3xl">Validity period</p>
              <p class="text-2xl text-gray-400">{this.props.selectedDatabase.map(s => s?.cycle.validityPeriod)}</p>
            </div>
          </div>
        </div>
      </div>
    )
  }
}
