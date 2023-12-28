import { NavigraphNavdataInterface } from "../js"
import { DATABASE_PATH } from "./constants"

const navdataInterface = new NavigraphNavdataInterface()

describe("test", () => {
  it("Set active database", async () => {
    await navdataInterface.setActiveDatabase(DATABASE_PATH)
  })

  it("Fetch airport", async () => {
    const airport = await navdataInterface.getAirport("KJFK")

    expect(airport.airport_identifier).toBe("KJFK")
  })
})
