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

class Capture
{
  private:
    pho::api::PhoXiFactory Factory;
    pho::api::PPhoXi PhoXiDevice;
    pho::api::PFrame LastFrame;

    void ConnectPhoXiDeviceBySerial(int argc, char *argv[]);
    void ChangeSettings(int argc, char *argv[]);
    void CaptureAndSaveFrame(int argc, char *argv[]);

  public:
    Capture(){};
    ~Capture() {
        if (PhoXiDevice && PhoXiDevice->isConnected()) {
            std::cout << "Capture object destructor: Disconnecting device." << std::endl;
            if (PhoXiDevice->isAcquiring()) {
                PhoXiDevice->StopAcquisition();
            }
            PhoXiDevice->Disconnect(true); 
        }
    };
    void Run(int argc, char *argv[]);
};

void Capture::ConnectPhoXiDeviceBySerial(int argc, char *argv[])
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
        std::cerr << "Connection to the device " << hardware_identification << " was Unsuccessful (PhoXiDevice is null)!" << std::endl;
        throw std::runtime_error("Failed to connect to device: " + hardware_identification + ". Factory.CreateAndConnect returned null.");
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
    // The FullAPIExample checks IsEnabled, CanSet, CanGet for the *entire* CapturingSettings group
    if (!PhoXiDevice->CapturingSettings.isEnabled() 
        || !PhoXiDevice->CapturingSettings.CanSet() 
        || !PhoXiDevice->CapturingSettings.CanGet())
    {
        std::cerr << "Warning: CapturingSettings group is not fully enabled or accessible. Attempting to set anyway." << std::endl;
        // Or throw: throw std::runtime_error("CapturingSettings group not fully accessible.");
    }
    
    pho::api::PhoXiCapturingSettings newCapturingSettings = PhoXiDevice->CapturingSettings;
    if (!PhoXiDevice->CapturingSettings.isLastOperationSuccessful()) { // Check after getting current settings
        throw std::runtime_error("Failed to get current CapturingSettings: " + std::string(PhoXiDevice->CapturingSettings.GetLastErrorMessage().c_str()));
    }


    newCapturingSettings.ShutterMultiplier = std::stoi(argv[6]);
    newCapturingSettings.ScanMultiplier = std::stoi(argv[7]);

    // Resolution setting (CapturingMode) is handled separately as it's not part of PhoXiCapturingSettings struct
    if (PhoXiDevice->SupportedCapturingModes.isEnabled() && PhoXiDevice->SupportedCapturingModes.CanGet() &&
        PhoXiDevice->CapturingMode.isEnabled() && PhoXiDevice->CapturingMode.CanSet())
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
        std::cerr << "Warning: Cannot get/set SupportedCapturingModes or CapturingMode feature. Resolution change skipped." << std::endl;
    }

    newCapturingSettings.CameraOnlyMode = static_cast<bool>(std::stoi(argv[9]));
    newCapturingSettings.AmbientLightSuppression = static_cast<bool>(std::stoi(argv[10]));
    newCapturingSettings.CodingStrategy = argv[11];
    newCapturingSettings.CodingQuality = argv[12];
    newCapturingSettings.TextureSource = argv[13];
    newCapturingSettings.SinglePatternExposure = std::stod(argv[14]);
    newCapturingSettings.MaximumFPS = std::stod(argv[15]);
    newCapturingSettings.LaserPower = std::stoi(argv[16]);
    
    // Direct assignment for ProjectionOffsetLeft, ProjectionOffsetRight, LedPower.
    // If these fields do not exist in your pho::api::PhoXiCapturingSettings struct,
    // this will cause a COMPILE-TIME ERROR.
    // If they exist but are not supported by the device, the API might ignore them,
    // or the PhoXiDevice->CapturingSettings = newCapturingSettings; call might fail
    // (indicated by isLastOperationSuccessful).
    newCapturingSettings.ProjectionOffsetLeft = std::stoi(argv[17]);
    newCapturingSettings.ProjectionOffsetRight = std::stoi(argv[18]);
    newCapturingSettings.LedPower = std::stoi(argv[19]);

    PhoXiDevice->CapturingSettings = newCapturingSettings;
    if (!PhoXiDevice->CapturingSettings.isLastOperationSuccessful())
    {
        throw std::runtime_error("Failed to set CapturingSettings (possibly due to unsupported individual parameters like ProjectionOffset/LedPower or other issues): " + std::string(PhoXiDevice->CapturingSettings.GetLastErrorMessage().c_str()));
    }
    

    // Processing Settings
    if (!PhoXiDevice->ProcessingSettings.isEnabled() 
        || !PhoXiDevice->ProcessingSettings.CanSet() 
        || !PhoXiDevice->ProcessingSettings.CanGet()) {
        std::cerr << "Warning: ProcessingSettings group is not fully enabled or accessible. Attempting to set anyway." << std::endl;
    }

    pho::api::PhoXiProcessingSettings newProcessingSettings = PhoXiDevice->ProcessingSettings;
    if (!PhoXiDevice->ProcessingSettings.isLastOperationSuccessful()){
         throw std::runtime_error("Failed to get current ProcessingSettings: " + std::string(PhoXiDevice->ProcessingSettings.GetLastErrorMessage().c_str()));
    }

    newProcessingSettings.Confidence = std::stod(argv[20]); 
    newProcessingSettings.SurfaceSmoothness = argv[21];
    newProcessingSettings.NormalsEstimationRadius = std::stoi(argv[22]);
    // API name from example is InterreflectionsFiltering (boolean)
    newProcessingSettings.InterreflectionsFiltering = static_cast<bool>(std::stoi(argv[23]));

    PhoXiDevice->ProcessingSettings = newProcessingSettings;
    if (!PhoXiDevice->ProcessingSettings.isLastOperationSuccessful())
    {
        throw std::runtime_error("Failed to set ProcessingSettings: " + std::string(PhoXiDevice->ProcessingSettings.GetLastErrorMessage().c_str()));
    }
    

    // Experimental Settings
    // The FullAPIExample does not show ExperimentalSettings, so access patterns are an educated guess.
    // Assuming it follows the same pattern as CapturingSettings/ProcessingSettings.
    if (PhoXiDevice->Features.IsSupported("ExperimentalSettings")) { // Keep this high-level check if available, otherwise remove
        if (!PhoXiDevice->ExperimentalSettings.isEnabled() 
            || !PhoXiDevice->ExperimentalSettings.CanSet() 
            || !PhoXiDevice->ExperimentalSettings.CanGet()) {
             std::cerr << "Warning: ExperimentalSettings group is not fully enabled or accessible. Attempting to set anyway." << std::endl;
        }

        pho::api::PhoXiExperimentalSettings newExperimentalSettings = PhoXiDevice->ExperimentalSettings;
        if (!PhoXiDevice->ExperimentalSettings.isLastOperationSuccessful()){
            std::cerr << "Warning: Failed to get current ExperimentalSettings: " << std::string(PhoXiDevice->ExperimentalSettings.GetLastErrorMessage().c_str()) << std::endl;
            // Not throwing an error here as these are experimental
        }

        // Direct assignment. COMPILE-TIME ERROR if these fields don't exist.
        // Runtime behavior depends on API/device if fields exist but are unsupported.
        newExperimentalSettings.AmbientLightSuppressionCompatibilityMode = static_cast<bool>(std::stoi(argv[24]));
        newExperimentalSettings.PatternDecompositionReach = argv[25];
        newExperimentalSettings.SignalContrastThreshold = std::stod(argv[26]);
        newExperimentalSettings.UseExtendedLogging = static_cast<bool>(std::stoi(argv[27]));
        
        PhoXiDevice->ExperimentalSettings = newExperimentalSettings;
        if (!PhoXiDevice->ExperimentalSettings.isLastOperationSuccessful())
        {
            // Don't throw for experimental, just warn
            std::cerr << "Warning: Failed to set some ExperimentalSettings: " << std::string(PhoXiDevice->ExperimentalSettings.GetLastErrorMessage().c_str()) << std::endl;
        }
    } else {
        std::cerr << "Warning: ExperimentalSettings feature group itself is not reported as supported by PhoXiDevice->Features.IsSupported(). Skipping ExperimentalSettings." << std::endl;
    }


    std::cout << "Settings changed attempt completed." << std::endl;
}

void Capture::CaptureAndSaveFrame(int argc, char *argv[])
{
    if (!PhoXiDevice || !PhoXiDevice->isConnected()) 
    {
        std::cerr << "Device is not created, or not connected for capture!" << std::endl;
        throw std::runtime_error("Device not connected during CaptureAndSaveFrame.");
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
    int FrameID = PhoXiDevice->TriggerFrame(true); // WaitForGrabbingEnd = true

    if (FrameID < 0)
    {
        std::cerr << "Trigger was unsuccessful! Error code: " << FrameID << std::endl;
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
        }
    }

    if (std::stoi(argv[4]) == 1)
    {
        if (LastFrame && !LastFrame->PointCloud.Empty()) 
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

void Capture::Run(int argc, char *argv[])
{
    try
    {
        ConnectPhoXiDeviceBySerial(argc, argv);
        ChangeSettings(argc, argv);
        CaptureAndSaveFrame(argc, argv);
    }
    catch (const std::runtime_error &e) 
    {
        std::cerr << std::endl
                  << "A runtime error occurred: " << e.what() << std::endl;
    }
    catch (const std::exception &e) 
    {
        std::cerr << std::endl 
                  << "A standard exception occurred: " << e.what() << std::endl;
    }
    catch (...) 
    {
        std::cerr << std::endl
                  << "An unknown non-standard exception occurred." << std::endl;
    }

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
            PhoXiDevice->Disconnect(true);
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
                  << " <hw_id> <scene_name> <save_praw> <save_ply> <save_tif> " 
                  << "<shutter_mult> <scan_mult> <res_idx> <cam_only> <ambient_suppress> " 
                  << "<coding_strat> <coding_qual> <texture_src> <single_pat_exp> <max_fps> " 
                  << "<laser_pow> <proj_off_L> <proj_off_R> <led_pow> " 
                  << "<max_inaccuracy> <surf_smooth> <norm_radius> <interreflect_filter> " 
                  << "<exp_als_compat> <exp_pat_decomp> <exp_sig_thresh> <exp_ext_log> " 
                  << "<praw_dir> <ply_dir> <tif_dir> <ip_address>" 
                  << std::endl;
        std::cerr << "Received " << argc - 1 << " arguments, expected 31." << std::endl;
        return 1;
    }

    std::cout << "Starting Capture Program with " << argc - 1 << " arguments." << std::endl;
    // Optional: Print all arguments for debugging
    // for(int i = 0; i < argc; ++i) {
    //     std::cout << "  argv[" << i << "]: " << argv[i] << std::endl;
    // }
    
    Capture Example;
    Example.Run(argc, argv);
    std::cout << "Capture Program finished." << std::endl;
    return 0;
}