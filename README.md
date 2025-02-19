# Setup
asdf
## Architecture
asdf
## Network setup
asdf
## Preparation
#### Overview: 
1. Aligning the origin coordinates. 

Motivation: Prepared CAD files come with origins that are placed in inconvenient positions, and often outside the actual component (as they have probably been exported in batch as parts of a bigger assembly). In order to properly visualize meshes and picking frames during detection, it is preferred that the origins are adjusted to match the main picking point and that the meshes used for vizualization have the same origin as the CAD files or meshes used for localization.
#### Step 1
A CAD software is needed here, Autocad, Freecad, or similar, I am for example using Auodesk Inventor. In Autodesk Inventor, create a new Assembly.

![alt text](instruction_images/preparation_1.png)
#### Step 2
Select the new assembly and place a component into it. This will enable us to align the component to the coordinate frame of the assembly and export the CAD with a newly defined origin.

![alt text](instruction_images/preparation_2.png)
#### Step 3
For example, let's prepare this "silver_gun".
![alt text](instruction_images/preparation_3.png)
#### Step 4
![alt text](instruction_images/preparation_4.png)
#### Step 5
![alt text](instruction_images/preparation_5.png)
#### Step 6
Selecting the assembly origin, we can visualize the origin planes of the assembly. We can do the sme things for the component we want to prepare, in this case "silver_gun".

![alt text](instruction_images/preparation_6.png)
#### Step 7
Let's say that we want to prepare the new origin coordinates to match the ones drawn on the figure below. The work now consists of aligning the frame of the assembly (shown in the bottom left of the figure) to match the drawn frame.

![alt text](instruction_images/preparation_7.png)
#### Step 8
We can use tools such as "Constrain" to align different planes. 

![alt text](instruction_images/preparation_8.png)
#### Step 9
For example, if we would like the Z coordinate to point "out" from the selected plane, we can use the flush constrain tool to align the selected plane and the "XY plane" of the assembly.

![alt text](instruction_images/preparation_9.png)
#### Step 10
The next figure shows that the target plane and XY plane are flushed, and that the Z coordinate is pointing outwards from the target plane as intended (lower left corner).

![alt text](instruction_images/preparation_10.png)
#### Step 11
Switching to the top view, we see that we first have to align the roration arounf the Z axis. 

![alt text](instruction_images/preparation_11.png)
#### Step 12
To do that, we cn again use the constrain tool to fluh the selected plane below with the YZ plane of the assembly.

![alt text](instruction_images/preparation_12.png)
#### Step 13
Switching again to the top view, we see that the angles are aligned as intended.

![alt text](instruction_images/preparation_13.png)
#### Step 14
The XY assembly plane is now locked to the first target plane, ensuring the good placement of the Z coordinate, and the YZ assembly plane is locked to the second target plane, ensuring a good angle of the Y and X coordinates. However, now we would like to move the origin of only the X and Y to the desired position, and to do so, we leave the XY flush locked, and supress the second flush with the YZ plane. This will allow us to move the component with its current oprientation in the XY plane.

![alt text](instruction_images/preparation_14.png)

#### Step 15
Like so, now we have the desired origin like we have drawn it above.

![alt text](instruction_images/preparation_15.png)
#### Step 16
Export the assembly now in a .stp CAD format for localization, and a .stl MESH file for visualization. 

![alt text](instruction_images/preparation_16.png)
![alt text](instruction_images/preparation_17.png)
#### Step 17
Start the Photoneo Localization software and import the prepared CAD file to verify the good position of the origin coordinate frame. The goal is to not manipulate the origin in the localization software, so all the coordinates and angles should be left as 0.

![alt text](instruction_images/preparation_18.png)

#### Step 18
Next, connect the Photoneo scanner and open the Phoxi Control software. Position the scanner so that the components are visible. Note that the scanning volume of the photoneo model S is quite small. Trigger a scan. 

![alt text](instruction_images/photoneo_s_range.png)
![alt text](instruction_images/scan_setup.jpg)
![alt text](instruction_images/scanning.jpg)
#### Step 19
The Phoxi control software should now visualize the point cloud. Make sure that the components to be localized are visible. If so, save the current scan, either in .ply or .praw. (.praw might hold more information, and it can be later converted to other formats if necessary, so .praw might be favored).

![alt text](instruction_images/preparation_19.png)
#### Step 20
Open the localization software and select the prepared CAD file. After inspecting that the origin is correct, go next while leaving the positions and orientations at 0. The software now allows a "live" scanner view, or a saved scan in order to test the localization and prepare a localization file (.plcf). Since we have saved a scan where the parts are visible, select "file" in the testing window and find the saved .praw file. The point cloud should now be loaded.

![alt text](instruction_images/preparation_20.png)
#### Step 21
Select a number od instances you would like to test for and start the localization. The localized items should now be highlightd and the overlap confidences displayed.

![alt text](instruction_images/preparation_21.png)
#### Step 22
Tweak the setting so see if the confidences can be increased and save the settings. This can be done per prepared item and later used as a parameter. TODO: Expland this and show how to.