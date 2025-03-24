#include <string>
#include <iostream>
#include <sstream>
#if defined(_WIN32)
    #include <windows.h>
#elif defined (__linux__)
    #include <unistd.h>
#endif

#include "PhoXi.h"

#if defined(_WIN32)
    #define LOCAL_CROSS_SLEEP(Millis) Sleep(Millis)
    #define DELIMITER "\\"
#elif defined (__linux__) || defined(__APPLE__)
    #define LOCAL_CROSS_SLEEP(Millis) usleep(Millis * 1000)
    #define DELIMITER "/"
#endif

class Connect
{
  private:
    pho::api::PhoXiFactory Factory;
    pho::api::PPhoXi PhoXiDevice;

    void ConnectPhoXiDeviceBySerial(int argc, char* argv[]);

    template<class T>
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
    Connect() {};
    ~Connect() {};
    void Run(int argc, char* argv[]);
};

void Connect::ConnectPhoXiDeviceBySerial(int argc, char* argv[])
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

void Connect::Run(int argc, char* argv[])
{
    try
    {
        ConnectPhoXiDeviceBySerial(argc, argv);
    }
    catch (std::runtime_error &InternalException)
    {
        std::cout << std::endl << "Exception was thrown: " << InternalException.what() << std::endl;
        if (PhoXiDevice->isConnected())
        {
            PhoXiDevice->Disconnect(true);
        }
    }
}

int main(int argc, char *argv[])
{
    Connect Example;
    Example.Run(argc, argv);
    return 0;
}

