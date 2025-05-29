#include <string>
#include <iostream>
#include <sstream>
#include <vector>    // Required for std::vector
#include <stdexcept> // Required for std::runtime_error

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

// ARGUMENTS (as per Rust code, 0-indexed in argv after program name):
// 0 - (program name)
// 1 - hardware_identification (string)
// 2 - scene_name (string)
// 3 - praw (bool as "0" or "1")
// 4 - ply (bool as "0" or "1")
// 5 - tif (bool as "0" or "1")
// 6 - capturing_settings::shutter_multiplier (int)
// 7 - capturing_settings::scan_multiplier (int)
// 8 - capturing_settings::resolution (index for SupportedCapturingModes, int)
// 9 - capturing_settings::camera_only_mode (bool as "0" or "1")
// 10 - capturing_settings::ambient_light_suppression (bool as "0" or "1")
// 11 - capturing_settings::coding_strategy (string)
// 12 - capturing_settings::coding_quality (string)
// 13 - capturing_settings::texture_source (string)
// 14 - capturing_settings::single_pattern_exposure (double)
// 15 - capturing_settings::maximum_fps (double)
// 16 - capturing_settings::laser_power (int)
// 17 - capturing_settings::projection_offset_left (int)
// 18 - capturing_settings::projection_offset_right (int)
// 19 - capturing_settings::led_power (int)
// 20 - processing_settings::max_inaccuracy (aka Confidence) (double)
// 21 - processing_settings::surface_smoothness (string)
// 22 - processing_settings::normals_estimation_radius (int)
// 23 - processing_settings::interreflections_filter (bool as "0" or "1")
// 24 - experimental_settings::ambient_light_suppression_compatibility_mode (bool as "0" or "1")
// 25 - experimental_settings::pattern_decomposition_reach (string)
// 26 - experimental_settings::signal_contrast_threshold (double)
// 27 - experimental_settings::use_extended_logging (bool as "0" or "1")
// 28 - praw_dir (string path)
// 29 - ply_dir (string path)
// 30 - tif_dir (string path)
// 31 - ip_identification (string, e.g., "192.168.1.27")

class Capture
{
  private:
    pho::api::PhoXiFactory Factory;
    pho::api::PPhoXi PhoXiDevice;
    pho::api::PFrame LastFrame;

    void ConnectPhoXiDeviceBySerial(int argc, char *argv[]);
    void ChangeSettings(int argc, char *argv[]);
    void CaptureAndSaveFrame(int argc, char *argv[]);

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
    Capture(){};
    ~Capture(){};
    void Run(int argc, char *argv[]);
};

void Capture::ConnectPhoXiDeviceBySerial(int argc, char *argv[])
{
    // Wait for the PhoXiControl
    while (!Factory.isPhoXiControlRunning())
    {
        std::cout << "Waiting for PhoXi Control..." << std::endl;
        LOCAL_CROSS_SLEEP(1000); // Increased sleep time
    }
    std::cout << "PhoXi Control Version: " << Factory.GetPhoXiControlVersion() << std::endl;
    std::cout << "PhoXi API Version: " << Factory.GetAPIVersion() << std::endl;

    pho::api::PhoXiTimeout Timeout = pho::api::PhoXiTimeout::Infinity; // Wait indefinitely for connection
    std::string hardware_identification = argv[1];
    std::cout << "Attempting to connect to device HW ID: " << hardware_identification << std::endl;
    // The IP from argv[31] is not used here, connection is by HW ID.
    // To connect by IP, you might use:
    // PhoXiDevice = Factory.CreateAndConnect("PHOXI_CONTROL_HOST_NAME:" + hardware_identification + "@" + std::string(argv[31]), Timeout);
    // Or other API-specific methods if available for direct IP connection.
    PhoXiDevice = Factory.CreateAndConnect(hardware_identification, Timeout);
    if (PhoXiDevice)
    {
        std::cout << "Connection to the device " << hardware_identification << " was Successful!" << std::endl;
        std::cout << "Connected device type: " << std::string(PhoXiDevice->GetType()) << std::endl;
    }
    else
    {
        std::cerr << "Connection to the device " << hardware_identification << " was Unsuccessful!" << std::endl;
        throw std::runtime_error("Failed to connect to device: " + hardware_identification);
    }
}

void Capture::ChangeSettings(int argc, char *argv[])
{
    if (!PhoXiDevice || !PhoXiDevice->isConnected())
    {
        std::cerr << "Device not connected. Cannot change settings." << std::endl;
        throw std::runtime_error("Device not connected during ChangeSettings.");
    }
    std::cout << "Changing settings..." << std::endl;

    // Capturing Settings
    if (PhoXiDevice->CapturingSettings.isEnabled() && PhoXiDevice->CapturingSettings.CanSet())
    {
        pho::api::PhoXiCapturingSettings newCapturingSettings = PhoXiDevice->CapturingSettings;

        newCapturingSettings.ShutterMultiplier = std::stoi(argv[6]);
        newCapturingSettings.ScanMultiplier = std::stoi(argv[7]);

        if (PhoXiDevice->SupportedCapturingModes.isEnabled() && PhoXiDevice->SupportedCapturingModes.CanGet())
        {
            std::vector<pho::api::PhoXiCapturingMode> SupportedCapturingModes = PhoXiDevice->SupportedCapturingModes;
            if (!PhoXiDevice->SupportedCapturingModes.isLastOperationSuccessful())
            {
                throw std::runtime_error("Failed to get SupportedCapturingModes: " + PhoXiDevice->SupportedCapturingModes.GetLastErrorMessage().c_str());
            }
            int resolutionIndex = std::stoi(argv[8]);
            if (resolutionIndex >= 0 && static_cast<size_t>(resolutionIndex) < SupportedCapturingModes.size())
            {
                PhoXiDevice->CapturingMode = SupportedCapturingModes[resolutionIndex];
                if (!PhoXiDevice->CapturingMode.isLastOperationSuccessful())
                {
                    throw std::runtime_error("Failed to set CapturingMode: " + PhoXiDevice->CapturingMode.GetLastErrorMessage().c_str());
                }
            }
            else
            {
                std::cerr << "Warning: Invalid resolution index: " << resolutionIndex << ". Max index: " << SupportedCapturingModes.size() -1 << ". Using current resolution." << std::endl;
            }
        }
        else
        {
            std::cerr << "Warning: Cannot get SupportedCapturingModes. Resolution change skipped." << std::endl;
        }

        newCapturingSettings.CameraOnlyMode = static_cast<bool>(std::stoi(argv[9]));
        newCapturingSettings.AmbientLightSuppression = static_cast<bool>(std::stoi(argv[10]));
        newCapturingSettings.CodingStrategy = argv[11];
        newCapturingSettings.CodingQuality = argv[12];
        newCapturingSettings.TextureSource = argv[13];
        newCapturingSettings.SinglePatternExposure = std::stod(argv[14]);
        newCapturingSettings.MaximumFPS = std::stod(argv[15]);
        newCapturingSettings.LaserPower = std::stoi(argv[16]);
        
        // Uncommented based on new argument list - verify API member names if issues arise
        if(PhoXiDevice->Features.IsSupported("CapturingSettings.ProjectionOffsetLeft")) newCapturingSettings.ProjectionOffsetLeft = std::stoi(argv[17]);
        if(PhoXiDevice->Features.IsSupported("CapturingSettings.ProjectionOffsetRight")) newCapturingSettings.ProjectionOffsetRight = std::stoi(argv[18]);
        if(PhoXiDevice->Features.IsSupported("CapturingSettings.LedPower")) newCapturingSettings.LedPower = std::stoi(argv[19]);


        PhoXiDevice->CapturingSettings = newCapturingSettings;
        if (!PhoXiDevice->CapturingSettings.isLastOperationSuccessful())
        {
            throw std::runtime_error("Failed to set CapturingSettings: " + PhoXiDevice->CapturingSettings.GetLastErrorMessage().c_str());
        }
    }
    else
    {
        std::cerr << "Warning: CapturingSettings are not enabled or cannot be set." << std::endl;
    }

    // Processing Settings
    if (PhoXiDevice->ProcessingSettings.isEnabled() && PhoXiDevice->ProcessingSettings.CanSet())
    {
        pho::api::PhoXiProcessingSettings newProcessingSettings = PhoXiDevice->ProcessingSettings;
        newProcessingSettings.Confidence = std::stod(argv[20]); // MaxInaccuracy
        newProcessingSettings.SurfaceSmoothness = argv[21];
        newProcessingSettings.NormalsEstimationRadius = std::stoi(argv[22]);
        // For InterReflectionsFilter, API name is InterreflectionsFiltering
        if(PhoXiDevice->Features.IsSupported("ProcessingSettings.InterreflectionsFiltering")) newProcessingSettings.InterreflectionsFiltering = static_cast<bool>(std::stoi(argv[23]));


        PhoXiDevice->ProcessingSettings = newProcessingSettings;
        if (!PhoXiDevice->ProcessingSettings.isLastOperationSuccessful())
        {
            throw std::runtime_error("Failed to set ProcessingSettings: " + PhoXiDevice->ProcessingSettings.GetLastErrorMessage().c_str());
        }
    }
    else
    {
        std::cerr << "Warning: ProcessingSettings are not enabled or cannot be set." << std::endl;
    }

    // Experimental Settings - verify API member names if issues arise
    if (PhoXiDevice->Features.IsSupported("ExperimentalSettings") && PhoXiDevice->ExperimentalSettings.isEnabled() && PhoXiDevice->ExperimentalSettings.CanSet())
    {
        pho::api::PhoXiExperimentalSettings newExperimentalSettings = PhoXiDevice->ExperimentalSettings;
        if(PhoXiDevice->Features.IsSupported("ExperimentalSettings.AmbientLightSuppressionCompatibilityMode")) newExperimentalSettings.AmbientLightSuppressionCompatibilityMode = static_cast<bool>(std::stoi(argv[24]));
        if(PhoXiDevice->Features.IsSupported("ExperimentalSettings.PatternDecompositionReach")) newExperimentalSettings.PatternDecompositionReach = argv[25];
        if(PhoXiDevice->Features.IsSupported("ExperimentalSettings.SignalContrastThreshold")) newExperimentalSettings.SignalContrastThreshold = std::stod(argv[26]);
        if(PhoXiDevice->Features.IsSupported("ExperimentalSettings.UseExtendedLogging")) newExperimentalSettings.UseExtendedLogging = static_cast<bool>(std::stoi(argv[27]));
        
        PhoXiDevice->ExperimentalSettings = newExperimentalSettings;
        if (!PhoXiDevice->ExperimentalSettings.isLastOperationSuccessful())
        {
            // Don't throw for experimental, just warn, as they might not always be critical or fully supported
            std::cerr << "Warning: Failed to set some ExperimentalSettings: " << PhoXiDevice->ExperimentalSettings.GetLastErrorMessage().c_str() << std::endl;
        }
    }
    else
    {
        std::cerr << "Warning: ExperimentalSettings are not supported, enabled or cannot be set." << std::endl;
    }

    std::cout << "Settings changed attempt completed." << std::endl;
}

void Capture::CaptureAndSaveFrame(int argc, char *argv[])
{
    if (!PhoXiDevice || !PhoXiDevice->isConnected())
    {
        std::cerr << "Device is not created, or not connected!" << std::endl;
        throw std::runtime_error("Device not connected during CaptureAndSaveFrame.");
    }

    // Ensure software trigger mode
    if (PhoXiDevice->TriggerMode != pho::api::PhoXiTriggerMode::Software)
    {
        std::cout << "Device is not in Software trigger mode." << std::endl;
        if (PhoXiDevice->isAcquiring())
        {
            std::cout << "Stopping acquisition to change trigger mode." << std::endl;
            if (!PhoXiDevice->StopAcquisition())
            {
                // Log error but try to continue changing trigger mode
                std::cerr << "Warning: Error in StopAcquisition: " << PhoXiDevice->GetLastErrorMessage().c_str() << std::endl;
            }
        }
        std::cout << "Switching to Software trigger mode." << std::endl;
        PhoXiDevice->TriggerMode = pho::api::PhoXiTriggerMode::Software;
        if (!PhoXiDevice->TriggerMode.isLastOperationSuccessful())
        {
            throw std::runtime_error("Failed to switch to Software trigger mode: " + PhoXiDevice->TriggerMode.GetLastErrorMessage().c_str());
        }
    }

    // Start acquisition if not already started
    if (!PhoXiDevice->isAcquiring())
    {
        std::cout << "Starting acquisition..." << std::endl;
        if (!PhoXiDevice->StartAcquisition())
        {
            throw std::runtime_error("Error in StartAcquisition: " + PhoXiDevice->GetLastErrorMessage().c_str());
        }
    }

    int ClearedFrames = PhoXiDevice->ClearBuffer();
    std::cout << ClearedFrames << " frames were cleared from the cyclic buffer." << std::endl;

    if (!PhoXiDevice->isAcquiring()) // Double check
    {
        std::cerr << "Device is not acquiring after attempting to start." << std::endl;
        throw std::runtime_error("Device not acquiring.");
    }

    std::cout << "Triggering frame..." << std::endl;
    int FrameID = PhoXiDevice->TriggerFrame(true /*WaitForGrabbingEnd*/);

    if (FrameID < 0)
    {
        std::cerr << "Trigger was unsuccessful! Error code: " << FrameID << std::endl;
        pho::api::PhoXiError lastError;
        PhoXiDevice->GetLastError(lastError);
        if (lastError.IsError())
        {
            std::cerr << "Detailed PhoXi Error for trigger: " << lastError.GetErrorMessage() << std::endl;
        }
        throw std::runtime_error("Frame trigger failed.");
    }
    else
    {
        std::cout << "Frame was triggered, Frame Id: " << FrameID << std::endl;
    }

    std::cout << "Waiting for frame " << FrameID << "..." << std::endl;
    pho::api::PFrame Frame = PhoXiDevice->GetSpecificFrame(FrameID, pho::api::PhoXiTimeout::Infinity); // Wait indefinitely

    if (Frame && !Frame->Empty())
    {
        LastFrame = Frame;
        std::cout << "Frame " << FrameID << " retrieved successfully." << std::endl;
        std::cout << "  Frame Resolution: " << Frame->Info.Resolution.Width << " x " << Frame->Info.Resolution.Height << std::endl;
    }
    else
    {
        std::cerr << "Failed to retrieve the frame or frame is empty!" << std::endl;
        throw std::runtime_error("Failed to retrieve frame " + std::to_string(FrameID));
    }

    std::string sceneName = argv[2];

    // Save PRAV file if requested
    if (std::stoi(argv[3]) == 1)
    {
        std::string prawsSaveDir = argv[28];
        const std::string lastFramePraw = prawsSaveDir + DELIMITER + sceneName + ".praw";
        std::cout << "Attempting to save frame as .praw to: " << lastFramePraw << std::endl;
        if (PhoXiDevice->SaveLastOutput(lastFramePraw, FrameID))
        {
            std::cout << "Saved frame as .praw to: " << lastFramePraw << std::endl;
        }
        else
        {
            std::cerr << "Could not save frame as .praw to: " << lastFramePraw << " !" << std::endl;
            pho::api::PhoXiError lastError;
            PhoXiDevice->GetLastError(lastError); // Check for specific error from SaveLastOutput
            if (lastError.IsError())
            {
                std::cerr << "Detailed PhoXi Error for PRAW save: " << lastError.GetErrorMessage() << std::endl;
            }
        }
    }

    // Save PLY file if requested
    if (std::stoi(argv[4]) == 1)
    {
        if (!LastFrame->PointCloud.Empty())
        {
            std::string plysSaveDir = argv[29];
            const std::string lastFramePly = plysSaveDir + DELIMITER + sceneName + ".ply";
            std::cout << "Attempting to save frame as .ply to: " << lastFramePly << std::endl;
            if (LastFrame->SaveAsPly(lastFramePly, true, true)) // Save textured, save binary
            {
                std::cout << "Saved frame as .ply to: " << lastFramePly << std::endl;
            }
            else
            {
                std::cerr << "Could not save frame as .ply to " << lastFramePly << " !" << std::endl;
            }
        }
        else
        {
            std::cerr << "Point cloud data is empty. Skipping PLY save for frame " << FrameID << "." << std::endl;
        }
    }

    // Save TIF file if requested
    if (std::stoi(argv[5]) == 1)
    {
        std::string tifsSaveDir = argv[30];
        const std::string lastFrameTif = tifsSaveDir + DELIMITER + sceneName + ".tif";
        std::cout << "Attempting to save frame as .tif to: " << lastFrameTif << std::endl;
        if (PhoXiDevice->SaveLastOutput(lastFrameTif, FrameID))
        {
            std::cout << "Saved frame as .tif to: " << lastFrameTif << std::endl;
        }
        else
        {
            std::cerr << "Could not save frame as .tif to: " << lastFrameTif << " !" << std::endl;
            pho::api::PhoXiError lastError;
            PhoXiDevice->GetLastError(lastError); // Check for specific error from SaveLastOutput
            if (lastError.IsError())
            {
                std::cerr << "Detailed PhoXi Error for TIF save: " << lastError.GetErrorMessage() << std::endl;
            }
        }
    }

    // Stop acquisition after saving and other operations for this frame
    std::cout << "Stopping acquisition after processing frame " << FrameID << "." << std::endl;
    if (PhoXiDevice->isAcquiring())
    {
        if (!PhoXiDevice->StopAcquisition())
        {
            std::cerr << "Warning: Error in StopAcquisition after saving: " << PhoXiDevice->GetLastErrorMessage().c_str() << std::endl;
        }
    }
}

void Capture::Run(int argc, char *argv[])
{
    try
    {
        ConnectPhoXiDeviceBySerial(argc, argv);
        ChangeSettings(argc, argv);
        CaptureAndSaveFrame(argc, argv);
    }
    catch (const pho::api::PhoXiException &PhoXiEx) // Catch PhoXi specific exceptions first
    {
        std::cerr << std::endl
                  << "PhoXi API Exception was thrown: " << PhoXiEx.what() << std::endl;
        if (PhoXiEx.GetError().IsError())
        {
            std::cerr << "Detailed PhoXi Error: " << PhoXiEx.GetError().GetErrorMessage() << std::endl;
        }
    }
    catch (const std::runtime_error &InternalException)
    {
        std::cerr << std::endl
                  << "Runtime Exception was thrown: " << InternalException.what() << std::endl;
    }
    catch (const std::exception &StdEx) // Catch other standard exceptions
    {
        std::cerr << std::endl
                  << "Standard Exception was thrown: " << StdEx.what() << std::endl;
    }
    catch (...) // Catch all other unknown exceptions
    {
        std::cerr << std::endl
                  << "An unknown exception occurred." << std::endl;
    }

    // Disconnect and logout
    if (PhoXiDevice) // Check if PhoXiDevice was ever initialized
    {
        if (PhoXiDevice->isConnected())
        {
            std::cout << "Disconnecting device..." << std::endl;
            if (PhoXiDevice->isAcquiring())
            { // Stop acquisition if it's still running
                PhoXiDevice->StopAcquisition();
            }
            PhoXiDevice->Disconnect(true /*logout device*/);
            std::cout << "Device disconnected." << std::endl;
        }
         PhoXiDevice = nullptr; // Release the smart pointer
    }
}

int main(int argc, char *argv[])
{
    // argc should be 32: program name (argv[0]) + 31 arguments (argv[1] to argv[31])
    if (argc < 32)
    {
        std::cerr << "Usage: " << argv[0]
                  << " <hw_id> <scene_name> <save_praw> <save_ply> <save_tif> " // 1-5
                  << "<shutter_mult> <scan_mult> <res_idx> <cam_only> <ambient_suppress> " // 6-10
                  << "<coding_strat> <coding_qual> <texture_src> <single_pat_exp> <max_fps> " // 11-15
                  << "<laser_pow> <proj_off_L> <proj_off_R> <led_pow> " // 16-19
                  << "<max_inaccuracy> <surf_smooth> <norm_radius> <interreflect_filter> " // 20-23
                  << "<exp_als_compat> <exp_pat_decomp> <exp_sig_thresh> <exp_ext_log> " // 24-27
                  << "<praw_dir> <ply_dir> <tif_dir> <ip_address>" // 28-31
                  << std::endl;
        std::cerr << "Received " << argc - 1 << " arguments, expected 31." << std::endl;
        return 1;
    }

    std::cout << "Starting Capture Program with " << argc - 1 << " arguments." << std::endl;
    Capture Example;
    Example.Run(argc, argv);
    std::cout << "Capture Program finished." << std::endl;
    return 0;
}