cmake_minimum_required(VERSION 3.5)
project(phoxi_control_interface_ros)

find_package(ament_cmake REQUIRED)

if(NOT DEFINED CMAKE_SUPPRESS_DEVELOPER_WARNINGS)
     set(CMAKE_SUPPRESS_DEVELOPER_WARNINGS 1 CACHE INTERNAL "No dev warnings")
endif()

include(r2r_cargo.cmake)

# put ros package dependencies here.
r2r_cargo(rcl                       # mandatory
          rcl_action                # mandatory
          rmw_fastrtps_cpp          # (needed to build with RMW_IMPLEMENTATION=rmw_fastrtps_cpp)
          FastRTPS                  # (needed to build with RMW_IMPLEMENTATION=rmw_fastrtps_cpp)     
          phoxi_control_msgs
          tf2_msgs                  # to publish TF messages to tf and tf_static
          std_msgs                  # the Header message is here 
          std_srvs
          sensor_msgs
          geometry_msgs             # the Transform and TransformStamped messages are here
         )

# install binaries
install(PROGRAMS
  ${CMAKE_SOURCE_DIR}/target/colcon/main
  DESTINATION lib/${PROJECT_NAME}
)

# we need this for ros/colcon
ament_package()