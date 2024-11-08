import {
  ComponentProps,
  DisplayComponent,
  FSComponent,
  MappedSubject,
  ObjectSubject,
  Subject,
  VNode,
} from "@microsoft/msfs-sdk"
import { NavigraphNavigationDataInterface } from "@navigraph/msfs-navigation-data-interface"
import { InterfaceNavbarItemV2, InterfaceSwitch } from "../../Utils"

interface TestPageProps extends ComponentProps {
  interface: NavigraphNavigationDataInterface
}

interface FunctionDescriptor {
  index: number
  arguments: string[]
  name: string
  functionCallback: (input?: string, inputAlt?: string) => unknown
}

interface InputState {
  active: boolean
  type: InputStateType
}

enum InputStateType {
  String,
  Bool,
}

export class TestPage extends DisplayComponent<TestPageProps> {
  private readonly selectedFunction = Subject.create(0)
  private readonly input1State = ObjectSubject.create<InputState>({
    active: false,
    type: InputStateType.String,
  })
  private readonly input2State = ObjectSubject.create<InputState>({
    active: false,
    type: InputStateType.String,
  })

  private strToBool(input?: string): boolean {
    return input == "true" ? true : false
  }

  private readonly functionList: FunctionDescriptor[] = [
    {
      index: 0,
      arguments: ["url: string"],
      name: "DownloadNavigationData",
      functionCallback: input => this.props.interface.download_navigation_data(input ?? ""),
    },
    {
      index: 1,
      arguments: [],
      name: "GetActivePackage",
      functionCallback: () => this.props.interface.get_active_package(),
    },
    {
      index: 2,
      arguments: ["sort: bool", "filter: bool"],
      name: "ListAvailablePackages",
      functionCallback: (input, inputAlt) =>
        this.props.interface.list_available_packages(this.strToBool(input), this.strToBool(inputAlt)),
    },
    {
      index: 3,
      arguments: ["uuid: string"],
      name: "SetActivePackage",
      functionCallback: input => this.props.interface.download_navigation_data(input ?? ""),
    },
  ]

  private readonly _generateInputs = this.selectedFunction.map(selectedFunction => {
    const functionObj = this.functionList[selectedFunction]

    const functionArgCount = functionObj.arguments.length

    switch (functionArgCount) {
      case 1: {
        this.input1State.set("active", true)
        this.input2State.set("active", false)
        break
      }
      case 2: {
        this.input1State.set("active", true)
        this.input2State.set("active", true)
        break
      }
      default: {
        this.input1State.set("active", false)
        this.input2State.set("active", false)
        break
      }
    }

    functionObj.arguments.forEach((value, index) => {
      const argumentType = value.includes("bool") ? InputStateType.Bool : InputStateType.String

      switch (index) {
        case 1: {
          this.input2State.set("type", argumentType)
          break
        }
        default: {
          this.input1State.set("type", argumentType)
          break
        }
      }
    })
  })

  render(): VNode {
    return (
      <div class="size-full flex flex-col">
        <p class="mb-8 text-4xl">Test</p>
        <div class="flex flex-row">
          <div class="w-1/3 flex flex-col">
            <div class="overflow-scroll flex-grow bg-ng-background-500">
              {this.functionList.map(obj => (
                <InterfaceNavbarItemV2
                  content={""}
                  class="w-full p-2 flex flex-col items-start"
                  activeClass="bg-blue-400"
                  active={this.selectedFunction.map(index => index === obj.index)}
                  setActive={() => this.selectedFunction.set(obj.index)}
                >
                  <p class="text-xl">{obj.name}</p>
                  <p class="text-lg">({obj.arguments.join(", ")})</p>
                </InterfaceNavbarItemV2>
              ))}
            </div>
            <div class="flex flex-row">
              <InterfaceSwitch
                active={this.input1State.map(obj => {
                  return obj.active ? (obj.type === InputStateType.String ? 0 : 1) : 2
                })}
                pages={[
                  [0, <p class="text-xl">String</p>],
                  [1, <p class="text-xl">Bool</p>],
                  [2, <p class="text-xl">None</p>],
                ]}
              />
              <InterfaceSwitch
                active={this.input2State.map(obj => {
                  return obj.active ? (obj.type === InputStateType.String ? 0 : 1) : 2
                })}
                pages={[
                  [0, <p class="text-xl">String</p>],
                  [1, <p class="text-xl">Bool</p>],
                  [2, <p class="text-xl">None</p>],
                ]}
              />
            </div>
          </div>
          <div class="w-2/3 bg-ng-background-700"></div>
        </div>
      </div>
    )
  }
}
