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

class ConnectIPv4
{
  private:
    pho::api::PhoXiFactory Factory;
    pho::api::PPhoXi PhoXiDevice;

    void ConnectPhoXiDeviceByIPAddress(int argc, char* argv[]);

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
    ConnectIPv4() {};
    ~ConnectIPv4() {};
    void Run(int argc, char* argv[]);
};

void ConnectIPv4::ConnectPhoXiDeviceByIPAddress(int argc, char* argv[])
{
    std::string deviceType;
    using PhoXiDeviceType = pho::api::PhoXiDeviceType;
    deviceType = static_cast<std::string>(PhoXiDeviceType(PhoXiDeviceType::PhoXiScanner));
    std::string HWIdentification = argv[1];
    std::string Ip = argv[2];
    PhoXiDevice = Factory.CreateAndConnect(HWIdentification, deviceType, Ip);
    if (PhoXiDevice)
    {
        std::cout << "Connection to the device " << HWIdentification << " at " << Ip << " was Successful!" << std::endl;
    }
    else
    {
        std::cout << "Connection to the device " << HWIdentification << " at " << Ip << " was Unsuccessful!" << std::endl;
    }
}

void ConnectIPv4::Run(int argc, char* argv[])
{
    try
    {
        ConnectPhoXiDeviceByIPAddress(argc, argv);
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
    ConnectIPv4 Example;
    Example.Run(argc, argv);
    return 0;
}

