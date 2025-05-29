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

#include "PhoXi.h" // Assuming PhoXi.h is correctly included

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

class CaptureNew
{
  private:
    pho::api::PhoXiFactory Factory;
    pho::api::PPhoXi PhoXiDevice;
    pho::api::PFrame LastFrame;

    void ConnectPhoXiDeviceBySerial(int argc, char *argv[]);
    void ChangeSettings(int argc, char *argv[]);
    void CaptureNewAndSaveFrame(int argc, char *argv[]);

    // ReadLine template and string specialization (not used in current flow but kept for potential future use)
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
    CaptureNew(){};
    ~CaptureNew() {
        // Ensure device is disconnected if object is destroyed
        // This acts as a fallback if Run() doesn't complete its cleanup
        if (PhoXiDevice && PhoXiDevice->isConnected()) {
            std::cout << "Capture object destructor: Disconnecting device." << std::endl;
            if (PhoXiDevice->isAcquiring()) {
                PhoXiDevice->StopAcquisition();
            }
            PhoXiDevice->Disconnect(true); // Logout
        }
    };
    void Run(int argc, char *argv[]);
};

void CaptureNew::ConnectPhoXiDeviceBySerial(int argc, char *argv[])
{
    while (!Factory.isPhoXiControlRunning())
    {
        std::cout << "Waiting for PhoXi Control..." << std::endl;
        LOCAL_CROSS_SLEEP(1000); 
    }
    std::cout << "PhoXi Control Version: " << Factory.GetPhoXiControlVersion() << std::endl;
    std::cout << "PhoXi API Version: " << Factory.GetAPIVersion() << std::endl;

    pho::api::PhoXiTimeout Timeout = pho::api::PhoXiTimeout::Infinity; 
    std::string hardware_identification = argv[1];
    std::cout << "Attempting to connect to device HW ID: " << hardware_identification << std::endl;
    
    PhoXiDevice = Factory.CreateAndConnect(hardware_identification, Timeout);
    if (PhoXiDevice)
    {
        std::cout << "Connection to the device " << hardware_identification << " was Successful!" << std::endl;
        std::cout << "Connected device type: " << std::string(PhoXiDevice->GetType()) << std::endl;
    }
    else
    {
        // The Factory.CreateAndConnect might return nullptr without setting a last error message on a PhoXiDevice object
        // It's better to throw a generic error or check Factory's error state if available
        std::cerr << "Connection to the device " << hardware_identification << " was Unsuccessful (PhoXiDevice is null)!" << std::endl;
        throw std::runtime_error("Failed to connect to device: " + hardware_identification + ". Factory.CreateAndConnect returned null.");
    }
}

void CaptureNew::ChangeSettings(int argc, char *argv[])
{
    if (!PhoXiDevice || !PhoXiDevice->isConnected()) // Check PhoXiDevice first
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
                throw std::runtime_error("Failed to get SupportedCapturingModes: " + std::string(PhoXiDevice->SupportedCapturingModes.GetLastErrorMessage().c_str()));
            }
            int resolutionIndex = std::stoi(argv[8]);
            if (resolutionIndex >= 0 && static_cast<size_t>(resolutionIndex) < SupportedCapturingModes.size())
            {
                PhoXiDevice->CapturingMode = SupportedCapturingModes[resolutionIndex];
                if (!PhoXiDevice->CapturingMode.isLastOperationSuccessful())
                {
                    throw std::runtime_error("Failed to set CapturingMode: " + std::string(PhoXiDevice->CapturingMode.GetLastErrorMessage().c_str()));
                }
            }
            else
            {
                std::cerr << "Warning: Invalid resolution index: " << resolutionIndex << ". Max index: " << (SupportedCapturingModes.empty() ? -1 : static_cast<int>(SupportedCapturingModes.size()) -1) << ". Using current resolution." << std::endl;
            }
        }
        else
        {
            std::cerr << "Warning: Cannot get SupportedCapturingModes or feature not enabled. Resolution change skipped." << std::endl;
        }

        newCapturingSettings.CameraOnlyMode = static_cast<bool>(std::stoi(argv[9]));
        newCapturingSettings.AmbientLightSuppression = static_cast<bool>(std::stoi(argv[10]));
        newCapturingSettings.CodingStrategy = argv[11];
        newCapturingSettings.CodingQuality = argv[12];
        newCapturingSettings.TextureSource = argv[13];
        newCapturingSettings.SinglePatternExposure = std::stod(argv[14]);
        newCapturingSettings.MaximumFPS = std::stod(argv[15]);
        newCapturingSettings.LaserPower = std::stoi(argv[16]);
        
        // For these, check if the feature name is correct as per API docs for your device model/firmware
        if(PhoXiDevice->Features.IsSupported("CapturingSettings.ProjectionOffsetLeft")) newCapturingSettings.ProjectionOffsetLeft = std::stoi(argv[17]);
        else std::cerr << "Warning: CapturingSettings.ProjectionOffsetLeft not supported or name incorrect." << std::endl;
        if(PhoXiDevice->Features.IsSupported("CapturingSettings.ProjectionOffsetRight")) newCapturingSettings.ProjectionOffsetRight = std::stoi(argv[18]);
        else std::cerr << "Warning: CapturingSettings.ProjectionOffsetRight not supported or name incorrect." << std::endl;
        if(PhoXiDevice->Features.IsSupported("CapturingSettings.LedPower")) newCapturingSettings.LedPower = std::stoi(argv[19]);
        else std::cerr << "Warning: CapturingSettings.LedPower not supported or name incorrect." << std::endl;

        PhoXiDevice->CapturingSettings = newCapturingSettings;
        if (!PhoXiDevice->CapturingSettings.isLastOperationSuccessful())
        {
            throw std::runtime_error("Failed to set CapturingSettings: " + std::string(PhoXiDevice->CapturingSettings.GetLastErrorMessage().c_str()));
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
        newProcessingSettings.Confidence = std::stod(argv[20]); 
        newProcessingSettings.SurfaceSmoothness = argv[21];
        newProcessingSettings.NormalsEstimationRadius = std::stoi(argv[22]);
        if(PhoXiDevice->Features.IsSupported("ProcessingSettings.InterreflectionsFiltering")) newProcessingSettings.InterreflectionsFiltering = static_cast<bool>(std::stoi(argv[23]));
        else std::cerr << "Warning: ProcessingSettings.InterreflectionsFiltering not supported or name incorrect." << std::endl;

        PhoXiDevice->ProcessingSettings = newProcessingSettings;
        if (!PhoXiDevice->ProcessingSettings.isLastOperationSuccessful())
        {
            throw std::runtime_error("Failed to set ProcessingSettings: " + std::string(PhoXiDevice->ProcessingSettings.GetLastErrorMessage().c_str()));
        }
    }
    else
    {
        std::cerr << "Warning: ProcessingSettings are not enabled or cannot be set." << std::endl;
    }

    // Experimental Settings
    if (PhoXiDevice->Features.IsSupported("ExperimentalSettings") && PhoXiDevice->ExperimentalSettings.isEnabled() && PhoXiDevice->ExperimentalSettings.CanSet())
    {
        pho::api::PhoXiExperimentalSettings newExperimentalSettings = PhoXiDevice->ExperimentalSettings;
        if(PhoXiDevice->Features.IsSupported("ExperimentalSettings.AmbientLightSuppressionCompatibilityMode")) newExperimentalSettings.AmbientLightSuppressionCompatibilityMode = static_cast<bool>(std::stoi(argv[24]));
        else std::cerr << "Warning: ExperimentalSettings.AmbientLightSuppressionCompatibilityMode not supported." << std::endl;
        if(PhoXiDevice->Features.IsSupported("ExperimentalSettings.PatternDecompositionReach")) newExperimentalSettings.PatternDecompositionReach = argv[25];
        else std::cerr << "Warning: ExperimentalSettings.PatternDecompositionReach not supported." << std::endl;
        if(PhoXiDevice->Features.IsSupported("ExperimentalSettings.SignalContrastThreshold")) newExperimentalSettings.SignalContrastThreshold = std::stod(argv[26]);
        else std::cerr << "Warning: ExperimentalSettings.SignalContrastThreshold not supported." << std::endl;
        if(PhoXiDevice->Features.IsSupported("ExperimentalSettings.UseExtendedLogging")) newExperimentalSettings.UseExtendedLogging = static_cast<bool>(std::stoi(argv[27]));
        else std::cerr << "Warning: ExperimentalSettings.UseExtendedLogging not supported." << std::endl;
        
        PhoXiDevice->ExperimentalSettings = newExperimentalSettings;
        if (!PhoXiDevice->ExperimentalSettings.isLastOperationSuccessful())
        {
            std::cerr << "Warning: Failed to set some ExperimentalSettings: " << std::string(PhoXiDevice->ExperimentalSettings.GetLastErrorMessage().c_str()) << std::endl;
        }
    }
    else
    {
        std::cerr << "Warning: ExperimentalSettings are not supported, enabled or cannot be set." << std::endl;
    }

    std::cout << "Settings changed attempt completed." << std::endl;
}

void CaptureNew::CaptureNewAndSaveFrame(int argc, char *argv[])
{
    if (!PhoXiDevice || !PhoXiDevice->isConnected()) 
    {
        std::cerr << "Device is not created, or not connected for capture!" << std::endl;
        throw std::runtime_error("Device not connected during CaptureNewAndSaveFrame.");
    }

    if (PhoXiDevice->TriggerMode != pho::api::PhoXiTriggerMode::Software)
    {
        std::cout << "Device is not in Software trigger mode." << std::endl;
        if (PhoXiDevice->isAcquiring())
        {
            std::cout << "Stopping acquisition to change trigger mode." << std::endl;
            if (!PhoXiDevice->StopAcquisition())
            {
                std::cerr << "Warning: Error in StopAcquisition before trigger mode change: " << std::string(PhoXiDevice->GetLastErrorMessage().c_str()) << std::endl;
            }
        }
        std::cout << "Switching to Software trigger mode." << std::endl;
        PhoXiDevice->TriggerMode = pho::api::PhoXiTriggerMode::Software;
        if (!PhoXiDevice->TriggerMode.isLastOperationSuccessful())
        {
            throw std::runtime_error("Failed to switch to Software trigger mode: " + std::string(PhoXiDevice->TriggerMode.GetLastErrorMessage().c_str()));
        }
    }

    if (!PhoXiDevice->isAcquiring())
    {
        std::cout << "Starting acquisition..." << std::endl;
        if (!PhoXiDevice->StartAcquisition())
        {
            throw std::runtime_error("Error in StartAcquisition: " + std::string(PhoXiDevice->GetLastErrorMessage().c_str()));
        }
    }

    int ClearedFrames = PhoXiDevice->ClearBuffer();
    std::cout << ClearedFrames << " frames were cleared from the cyclic buffer." << std::endl;

    if (!PhoXiDevice->isAcquiring()) 
    {
        std::cerr << "Device is not acquiring after attempting to start." << std::endl;
        throw std::runtime_error("Device not acquiring.");
    }

    std::cout << "Triggering frame..." << std::endl;
    int FrameID = PhoXiDevice->TriggerFrame(true);

    if (FrameID < 0)
    {
        std::cerr << "Trigger was unsuccessful! Error code: " << FrameID << std::endl;
        // PhoXiDevice->GetLastError might provide more info specific to the device context,
        // but TriggerFrame itself returning < 0 is the primary indicator from the API example.
        // The GetLastErrorMessage on the TriggerMode feature might not be relevant here if the mode was set successfully earlier.
        throw std::runtime_error("Frame trigger failed with code: " + std::to_string(FrameID));
    }
    else
    {
        std::cout << "Frame was triggered, Frame Id: " << FrameID << std::endl;
    }

    std::cout << "Waiting for frame " << FrameID << "..." << std::endl;
    pho::api::PFrame Frame = PhoXiDevice->GetSpecificFrame(FrameID, pho::api::PhoXiTimeout::Infinity); 

    if (Frame && !Frame->Empty())
    {
        LastFrame = Frame;
        std::cout << "Frame " << FrameID << " retrieved successfully." << std::endl;
        std::cout << "  Frame Resolution: " << Frame->Info.Resolution.Width << " x " << Frame->Info.Resolution.Height << std::endl;
    }
    else
    {
        std::cerr << "Failed to retrieve the frame " << FrameID << " or frame is empty!" << std::endl;
        throw std::runtime_error("Failed to retrieve frame " + std::to_string(FrameID));
    }

    std::string sceneName = argv[2];

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
            std::cerr << "Could not save frame as .praw to: " << lastFramePraw << " ! Check PhoXi Control logs and ensure path is writable." << std::endl;
             // PhoXiDevice->GetLastError() might be useful here if the API supports it after SaveLastOutput
        }
    }

    if (std::stoi(argv[4]) == 1)
    {
        if (LastFrame && !LastFrame->PointCloud.Empty()) // Ensure LastFrame is valid
        {
            std::string plysSaveDir = argv[29];
            const std::string lastFramePly = plysSaveDir + DELIMITER + sceneName + ".ply";
            std::cout << "Attempting to save frame as .ply to: " << lastFramePly << std::endl;
            if (LastFrame->SaveAsPly(lastFramePly, true, true)) 
            {
                std::cout << "Saved frame as .ply to: " << lastFramePly << std::endl;
            }
            else
            {
                std::cerr << "Could not save frame as .ply to " << lastFramePly << " ! Ensure path is writable." << std::endl;
            }
        }
        else
        {
            std::cerr << "Point cloud data is empty or LastFrame is invalid. Skipping PLY save for frame " << FrameID << "." << std::endl;
        }
    }

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
            std::cerr << "Could not save frame as .tif to: " << lastFrameTif << " ! Check PhoXi Control logs and ensure path is writable." << std::endl;
        }
    }

    std::cout << "Stopping acquisition after processing frame " << FrameID << "." << std::endl;
    if (PhoXiDevice->isAcquiring())
    {
        if (!PhoXiDevice->StopAcquisition())
        {
            std::cerr << "Warning: Error in StopAcquisition after saving: " << std::string(PhoXiDevice->GetLastErrorMessage().c_str()) << std::endl;
        }
    }
}

void CaptureNew::Run(int argc, char *argv[])
{
    try
    {
        ConnectPhoXiDeviceBySerial(argc, argv);
        ChangeSettings(argc, argv);
        CaptureNewAndSaveFrame(argc, argv);
    }
    catch (const std::runtime_error &e) 
    {
        std::cerr << std::endl
                  << "A runtime error occurred: " << e.what() << std::endl;
    }
    catch (const std::exception &e) // Catch other std::exceptions (e.g., from std::stoi)
    {
        std::cerr << std::endl 
                  << "A standard exception occurred: " << e.what() << std::endl;
    }
    catch (...) // Catch-all for any other non-standard exceptions
    {
        std::cerr << std::endl
                  << "An unknown non-standard exception occurred." << std::endl;
    }

    // Final cleanup
    if (PhoXiDevice) 
    {
        if (PhoXiDevice->isConnected())
        {
            std::cout << "Run completed or aborted. Disconnecting device..." << std::endl;
            if (PhoXiDevice->isAcquiring())
            { 
                std::cout << "Device is acquiring, attempting to stop acquisition before final disconnect." << std::endl;
                if(!PhoXiDevice->StopAcquisition()){
                     std::cerr << "Warning: Failed to stop acquisition during final disconnect: " << std::string(PhoXiDevice->GetLastErrorMessage().c_str()) << std::endl;
                }
            }
            PhoXiDevice->Disconnect(true /*logout device*/);
            std::cout << "Device disconnected." << std::endl;
        }
         PhoXiDevice = nullptr; 
    }
}

int main(int argc, char *argv[])
{
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

    std::cout << "Starting CaptureNew Program with " << argc - 1 << " arguments." << std::endl;
    for(int i = 0; i < argc; ++i) {
        std::cout << "  argv[" << i << "]: " << argv[i] << std::endl;
    }
    
    CaptureNew Example;
    Example.Run(argc, argv);
    std::cout << "CaptureNew Program finished." << std::endl;
    return 0;
}