#include <string>
#include <cstdlib>
#include <iostream>
#include <sstream>
#include <iomanip>
#include <algorithm>
#include <cctype>

#if defined(_WIN32)
#include <windows.h>
#elif defined(__linux__)
#include <unistd.h>
#endif

#include <PhoLocalization.h>

using namespace pho::sdk;

// ARGUMENTS:
// 1 - hardware_identification
// 2 - scene_name
// 3 - target_name
// 4 - source_format
// 5 - stop_at_timeout
// 6 - stop_at_number
// 7 - scene noise reduction
// 8 - smart memory
// 9 - scene clustering level
// 10 - scene minimal cluster size
// 11 - scene maximal cluster size
// 12 - matching algorithm
// 13 - model keypoints sampling
// 14 - local search radius
// 15 - feature fit consideration level
// 16 - global maximal feature fit overflow
// 17 - fine alignment iterations
// 18 - fine alignment point set
// 19 - fine alignment point set sampling
// 20 - projection tolerance
// 21 - projection hidden part tolerance
// 22 - overlap
// 23 - praws location
// 24 - plys location
// 25 - plcfs location

bool to_bool(std::string str) {
	std::transform(str.begin(), str.end(), str.begin(), ::tolower);
	std::istringstream is(str);
	bool b;
	is >> std::boolalpha >> b;
	return b;
}

int main(int argc, char* argv[]) {
    std::unique_ptr<PhoLocalization> localization;

    std::string PrawsInputFolder = argv[23];
    std::string PlysInputFolder = argv[24];
	std::string PlcfsInputFolder = argv[25];

    try {
        localization.reset(new PhoLocalization());
    } catch (const AuthenticationException &ex) {
        std::cout << ex.what() << std::endl;
        return EXIT_FAILURE;
    }

    try {
        SceneSource scene;
		//std::cout << "Scene source format: " << argv[4] << std::endl;
        if (std::string(argv[4]) == "praw")
        {
			scene = SceneSource::File(PrawsInputFolder + "\\" + std::string(argv[2]) + ".praw");
			//std::cout << "Set praw: " << PrawsInputFolder + "\\" + std::string(argv[2]) + ".praw" << std::endl;
        }
		if (std::string(argv[4]) == "ply")
        {
            scene = SceneSource::File(PlysInputFolder + "\\" + argv[2] + ".ply");
			//std::cout << "Set ply: " << PlysInputFolder + "\\" + argv[2] + ".ply" << std::endl;
        }
		if (std::string(argv[4]) == "live")
        {
			scene = SceneSource::PhoXi(std::string(argv[1]));
        }
        localization->SetSceneSource(scene);
    } catch (const PhoLocalizationException &ex) {
        std::cout << "SceneSource Error: " << ex.what() << std::endl;
        return EXIT_FAILURE;
    }

    try {
		localization->LoadLocalizationConfiguration(PlcfsInputFolder + "\\" + std::string(argv[3]) + ".plcf");
		//std::cout << "Set plcf file: " << PlcfsInputFolder + "\\" + std::string(argv[3]) + ".plcf" << std::endl;
    } catch (const IOException &ex) {
		//std::cout << "Error loading plcf file: " << PlcfsInputFolder + "\\" + argv[3] + ".plcf" << std::endl;
        std::cout << "Error loading plcf file: " << ex.what() << std::endl;
        return EXIT_FAILURE;
    }

    localization->ClearStopCriteria();
    localization->SetStopCriterion(StopCriterion::Timeout(std::stoi(argv[5])));
    localization->SetStopCriterion(StopCriterion::NumberOfResults(std::stoi(argv[6])));

    localization->setSetting("Scene Noise Reduction", to_bool(std::string(argv[7])));
	localization->setSetting("Smart Memory", to_bool(std::string(argv[8])));
    localization->setSetting("Scene Clustering Level", std::string(argv[9]));
    localization->setSetting("Scene Minimal Cluster Size", std::stoi(argv[10]));
    localization->setSetting("Scene Maximal Cluster Size", std::stoi(argv[11]));
    localization->setSetting("Matching Algorithm", std::string(argv[12]));
    localization->setSetting("Model Keypoints Sampling", std::string(argv[13]));
    localization->setSetting("Local Search Radius", std::string(argv[14]));

	// These are problematic, don't know how to se them, what is their type? 
	//localization->setSetting("Feature Fit Consideration Level", sscanf(argv[15], "%zu"));
    //localization->setSetting("Global Maximal Feature Fit Overflow", std::stod(argv[16]));
    //localization->setSetting("Fine Alignment Iterations", std::stoi(argv[17]));
    
	localization->setSetting("Fine Alignment Point Set", std::string(argv[18]));
    localization->setSetting("Fine Alignment Point Set Sampling", std::string(argv[19]));
    localization->setSetting("Projection Tolerance", std::stoi(argv[20]));
    localization->setSetting("Projection Hidden Part Tolerance", std::stoi(argv[21]));
    localization->setSetting("Overlap", std::stod(argv[22]));

    AsynchroneResultQueue queue;
    try {
        queue = localization->StartAsync();
    } catch (const PhoLocalizationException &ex) {
        std::cout << "Error starting localization: " << ex.what() << std::endl;
        return EXIT_FAILURE;
    }

    TransformationMatrix4x4 result;
    std::cout << "Localization results:" << std::endl;
	std::size_t i = 0;
    while (queue.GetNext(result)) {
		std::cout << "RESULT " + std::to_string(i) + ": " << result << std::endl;
		i++;
    }
    std::cout << "Localization finished" << std::endl;

    return EXIT_SUCCESS;
}