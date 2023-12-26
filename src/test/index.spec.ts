import { NavigraphNavdataInterface } from "../js"

describe("test", () => {
  it("test1", async () => {
    console.log("running test 1")
    const navdataInterface = new NavigraphNavdataInterface()

    await navdataInterface.downloadNavdata(
      "https://packages.fmsdata.api.navigraph.com/0adb50e3-3c3e-4115-8324-e9fb625f03f1/e_dfd_2312.zip?sub=d2f9985c-1653-4052-ad35-68e5301c7428&Expires=1703632854&Key-Pair-Id=APKAJO4CE5J24DMH7HHA&Signature=YJqice6HmekirYCXFdY7O8ptpqzeiMNr-S1AvHYEBT6oRN8XdB4SqnAfT7Mn9uGptnsShTI4Flt0ZfA~FaEg-ogKFjnjehZZ23kbi6vnzxASgP1Ai-DdwJZ~mgrbdfmpBZh1vi1-FJKp043r8zPQII7S5kMxH7sZM4npBkKl-XFBOnfp7gmglfUXhEyudYLYZbG2uozCt9MystKrCRh7SoWIqbK7vnQxjcxpn3QHYwH5DxTcimYGhHQKT26Dvl77um6fADPXxfv8sTEEfaAwxx4u~yC1ByfjrsPJDqdRLKL0RScW9WG-1Ku-yIhoYuTic~qn9Kg9KYqonajwA6ntIQ__",
      "test",
    )
    await navdataInterface.setActiveDatabase("test")
    const airport = await navdataInterface.getAirport("KJFK")

    expect(airport.airport_identifier).toBe("KJFK")
    console.log("Finished test 1")
  })

  it("test2", async () => {
    console.log("running test 2")
    const navdataInterface = new NavigraphNavdataInterface()

    await navdataInterface.downloadNavdata(
      "https://packages.fmsdata.api.navigraph.com/0adb50e3-3c3e-4115-8324-e9fb625f03f1/e_dfd_2312.zip?sub=d2f9985c-1653-4052-ad35-68e5301c7428&Expires=1703632854&Key-Pair-Id=APKAJO4CE5J24DMH7HHA&Signature=YJqice6HmekirYCXFdY7O8ptpqzeiMNr-S1AvHYEBT6oRN8XdB4SqnAfT7Mn9uGptnsShTI4Flt0ZfA~FaEg-ogKFjnjehZZ23kbi6vnzxASgP1Ai-DdwJZ~mgrbdfmpBZh1vi1-FJKp043r8zPQII7S5kMxH7sZM4npBkKl-XFBOnfp7gmglfUXhEyudYLYZbG2uozCt9MystKrCRh7SoWIqbK7vnQxjcxpn3QHYwH5DxTcimYGhHQKT26Dvl77um6fADPXxfv8sTEEfaAwxx4u~yC1ByfjrsPJDqdRLKL0RScW9WG-1Ku-yIhoYuTic~qn9Kg9KYqonajwA6ntIQ__",
      "test",
    )
    await navdataInterface.setActiveDatabase("test")
    const airport = await navdataInterface.getAirport("KJFK")

    expect(airport.airport_identifier).toBe("KJFK")
    console.log("Finished test 2")
  })
})
