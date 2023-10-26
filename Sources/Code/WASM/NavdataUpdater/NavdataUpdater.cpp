// NavdataUpdater.cpp
#include <MSFS\MSFS.h>
#include <MSFS\Legacy\gauges.h>
#include <MSFS\MSFS_Render.h>
#include <MSFS\MSFS_CommBus.h>
#include <SimConnect.h>

#include "rapidjson/document.h"

#include <map>
#include <string>
#include <sys/stat.h>

static void DownloadNavdata(const char* jsonArgs, unsigned int size, void* ctx)
{
    rapidjson::Document document;
    document.Parse(jsonArgs);

    mkdir("\\work\\Navdata", -1);
    
    FILE* f = fopen("\\work\\Navdata\\navdata.json", "w");
    if (f)
    {
        fwrite(jsonArgs, sizeof(char), size, f);
        fclose(f);
    }
    fsCommBusCall("NavdataUpdaterReceived", nullptr, 0, FsCommBusBroadcast_JS);
}

extern "C"
{
    MSFS_CALLBACK bool NavdataUpdater_gauge_callback(FsContext ctx, int service_id, void* pData)
	{

		switch (service_id)
		{
		case PANEL_SERVICE_PRE_INSTALL:
		{
			sGaugeInstallData* p_install_data = (sGaugeInstallData*)pData;
			// Width given in p_install_data->iSizeX
			// Height given in p_install_data->iSizeY
			return true;
		}
		break;
		case PANEL_SERVICE_POST_INSTALL:
		{
			return true;
		}
		break;
		case PANEL_SERVICE_POST_INITIALIZE:
		{
			fsCommBusRegister("DownloadNavdata", DownloadNavdata);

			return true;
		}
		break;
		case PANEL_SERVICE_PRE_DRAW:
		{
			return true;
		}
		break;
		case PANEL_SERVICE_PRE_UPDATE:
		{
		}
		break;
		case PANEL_SERVICE_PRE_KILL:
		{
			return true;
		}
		break;
		}
		return false;
	}
}
