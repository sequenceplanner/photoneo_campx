#include <string>
#include <iostream>
#include <sstream>
#if defined(_WIN32)
#include <windows.h>
#elif defined(__linux__)
#include <unistd.h>
#endif

#include "PhoXi.h"

#if defined(_WIN32)
#define LOCAL_CROSS_SLEEP(Millis) Sleep(Millis)
#define DELIMITER "\\"
#elif defined(__linux__) || defined(__APPLE__)
#define LOCAL_CROSS_SLEEP(Millis) usleep(Millis * 1000)
#define DELIMITER "/"
#endif

// ARGUMENTS:
// 1 - hardware_identification
// 2 - scene_name
// 3 - praw
// 4 - ply
// 5 - tif
// 6 - capturing_settings::shutter_multiplier
// 7 - capturing_settings::scan_multiplier
// 8 - capturing_settings::resolution
// 9 - capturing_settings::camera_only_mode
// 10 - capturing_settings::ambient_light_suppression
// 11 - capturing_settings::coding_strategy 
// 12 - capturing_settings::coding_quality
// 13 - capturing_settings::texture_source
// 14 - capturing_settings::single_pattern_exposure
// 15 - capturing_settings::maximum_fps
// 16 - capturing_settings::laser_power
// 17 - capturing_settings::projection_offset_left
// 18 - capturing_settings::projection_offset_right
// 19 - capturing_settings::led_power
// 20 - procesing_settings::max_inaccuracy
// 21 - procesing_settings::surface_smoothness
// 22 - procesing_settings::normals_estimation_radius
// 23 - procesing_settings::interreflections_filter
// 24 - experimental_settings::ambient_light_suppression_compatibility_mode
// 25 - experimental_settings::pattern_decomposition_reach
// 26 - experimental_settings::signal_contrast_threshold
// 27 - experimental_settings::use_extended_logging

class Freerun
{
private:
    pho::api::PhoXiFactory Factory;
    pho::api::PPhoXi PhoXiDevice;
    pho::api::PFrame LastFrame;

    std::string trueString = "true";
    std::string falseString = "false";

    std::string FreerunOutputFolder = "C:\\Users\\photoneo\\Desktop\\scans\\freerun";

    void ConnectPhoXiDeviceBySerial(int argc, char *argv[]);
    void ChangeSettings(int argc, char* argv[]);
    void StartFreerun();

    template <class T>
    bool ReadLine(T &Output) const
    {
        std::string Input;
        std::getline(std::cin, Input);
        std::stringstream InputSteam(Input);
        return (InputSteam >> Output) ? true : false;
    }
    bool ReadLine(std::string &Output) const
    {
        std::getline(std::cin, Output);
        return true;
    }

public:
    Freerun(){};
    ~Freerun(){};
    void Run(int argc, char *argv[]);
};

void Freerun::ConnectPhoXiDeviceBySerial(int argc, char *argv[])
{
    pho::api::PhoXiTimeout Timeout = pho::api::PhoXiTimeout::ZeroTimeout;
    std::string hardware_identification = argv[1];
    PhoXiDevice = Factory.CreateAndConnect(hardware_identification, Timeout);
    if (PhoXiDevice)
    {
        std::cout << "Connection to the device " << hardware_identification << " was Successful!" << std::endl;
    }
    else
    {
        std::cout << "Connection to the device " << hardware_identification << " was Unsuccessful!" << std::endl;
    }
}

void Freerun::ChangeSettings(int argc, char* argv[])
{
    PhoXiDevice->CapturingSettings->ShutterMultiplier = std::stod(argv[6]);
    PhoXiDevice->CapturingSettings->ScanMultiplier = std::stoi(argv[7]);
    std::vector<pho::api::PhoXiCapturingMode> SupportedCapturingModes = PhoXiDevice->SupportedCapturingModes;
    if (!PhoXiDevice->SupportedCapturingModes.isLastOperationSuccessful())
    {
        throw std::runtime_error(PhoXiDevice->SupportedCapturingModes.GetLastErrorMessage().c_str());
    }
    PhoXiDevice->CapturingMode = SupportedCapturingModes[std::stoi(argv[8])];
    PhoXiDevice->CapturingSettings->CameraOnlyMode = std::stoi(argv[9]);
    PhoXiDevice->CapturingSettings->AmbientLightSuppression = std::stoi(argv[10]);
    PhoXiDevice->CapturingSettings->CodingStrategy = argv[11];
    PhoXiDevice->CapturingSettings->CodingQuality = argv[12];
    PhoXiDevice->CapturingSettings->TextureSource = argv[13];
    PhoXiDevice->CapturingSettings->SinglePatternExposure = std::stod(argv[14]);
    PhoXiDevice->CapturingSettings->MaximumFPS = std::stod(argv[15]);
    PhoXiDevice->CapturingSettings->LaserPower = std::stoi(argv[16]);
    // Are the following unsupported or what?
    //PhoxiDevice->CapturingSettings->ProjectionOffsetLeft = std::stoi(argv[17]);
    // PhoxiDevice->CapturingSettings->ProjectionOffsetRight = std::stoi(argv[18]);
    // PhoXiDevice->CapturingSettings->LedPower = std::stoi(argv[19]);
    PhoXiDevice->ProcessingSettings->Confidence = std::stod(argv[20]);
    PhoXiDevice->ProcessingSettings->SurfaceSmoothness = argv[21];
    PhoXiDevice->ProcessingSettings->NormalsEstimationRadius = std::stoi(argv[22]);
    // Are the following unsupported or what?
    // PhoXiDevice->ProcessingSettings->InterReflectionsFilter = argv[23];
    // PhoxiDevice->ExperimentalSettings->AmbientLightSuppressionCompatibilityMode = std::stoi(argv[24]);
    // PhoxiDevice->ExperimentalSettings->PatternDecompositionReach = argv[25];
    // PhoxiDevice->ExperimentalSettings->SignalContrastThreshold = std::stod(argv[26]);
    // PhoxiDevice->ExperimentalSettings->UseExtendedLogging = std::stoi(argv[24]);
}

void Freerun::StartFreerun()
{
    if (!PhoXiDevice || !PhoXiDevice->isConnected())
    {
        std::cout << "Device is not created, or not connected!" << std::endl;
        return;
    }
    if (PhoXiDevice->TriggerMode != pho::api::PhoXiTriggerMode::Freerun)
    {
        std::cout << "Device is not in Freerun mode" << std::endl;
        if (PhoXiDevice->isAcquiring())
        {
            std::cout << "Stopping acquisition" << std::endl;
            if (!PhoXiDevice->StopAcquisition())
            {
                throw std::runtime_error("Error in StopAcquistion");
            }
        }
        std::cout << "Switching to Freerun mode " << std::endl;
        PhoXiDevice->TriggerMode = pho::api::PhoXiTriggerMode::Freerun;
        if (!PhoXiDevice->TriggerMode.isLastOperationSuccessful())
        {
            throw std::runtime_error(PhoXiDevice->TriggerMode.GetLastErrorMessage().c_str());
        }
    }

    if (!PhoXiDevice->isAcquiring())
    {
        if (!PhoXiDevice->StartAcquisition())
        {
            throw std::runtime_error("Error in StartAcquisition");
        }
    }

    int ClearedFrames = PhoXiDevice->ClearBuffer();
    std::cout << ClearedFrames << " were cleared from the cyclic buffer" << std::endl;

    if (!PhoXiDevice->isAcquiring())
    {
        std::cout << "Device is not acquiring" << std::endl;
        return;
    }
    
    std::size_t i = 0;
    while (1)
    {
        pho::api::PFrame Frame = PhoXiDevice->GetFrame();
        //if (Frame)
        //{
        //    i++;
        //    std::cout << "Got a frame!";
        //    const auto freerunOutputFolder = FreerunOutputFolder.empty() ? std::string() : FreerunOutputFolder + DELIMITER;
        //    const auto lastFrameFreerun = freerunOutputFolder + std::to_string(i) + ".tif";
        //    if (PhoXiDevice->SaveLastOutput(lastFrameFreerun))
        //    {
        //        std::cout << "Saved frame as tif to: " << lastFrameFreerun << std::endl;
        //    }
        //    else
        //    {
        //        std::cout << "Could not save frame as tif to: " << lastFrameFreerun << " !" << std::endl;
        //    }
        //}
        //else
        // {
        //    std::cout << "Failed to retrieve the frame!";
        //}
    }
}

void Freerun::Run(int argc, char *argv[])
{
    try
    {
        ConnectPhoXiDeviceBySerial(argc, argv);
        ChangeSettings(argc, argv);
        StartFreerun();
    }
    catch (std::runtime_error &InternalException)
    {
        std::cout << std::endl
                  << "Exception was thrown: " << InternalException.what() << std::endl;
        if (PhoXiDevice->isConnected())
        {
            PhoXiDevice->Disconnect(true);
        }
    }
}

int main(int argc, char *argv[])
{
    Freerun Example;
    Example.Run(argc, argv);
    return 0;
}