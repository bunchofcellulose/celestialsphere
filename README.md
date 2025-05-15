# Celestial Sphere

Celestial Sphere is a web application that provides a neat and user-friendly interface for drawing and visualizing diagrams on a sphere. My aim for building this is to provide a tool for students and educators to easily learn and engage with spherical diagrams, which are commonly used in spherical astronomy.

## Controls

- Left Click on the sphere to add a point.
- Left Click on a point to select/deselect it.
- `Shift` + Left Click to select multiple points.
- Move a point by dragging it with left mouse button while it is selected.
- `Esc` to deselect the selected point(s).
- `Delete` to remove selected point(s).
- Name a point by typing when it's selected.
- Right click another point while having a point selected to draw/remove an arc of a great circle between them.
- Scroll to zoom in/out.
- Pan while holding the middle mouse button to rotate the sphere.
- `/` to place a point diametrically opposite to the selected point(s).
- `.` to draw a great circle having the selected point as a pole.
- `Shift` + type to name the great circle, while the associated pole point is selected.
- `Shift` + move to snap a point onto a nearby great circle.
- `Shift` + Left click on the sphere to add a point on a nearby great circle.

## Features

- The rotation of the sphere can be controlled by dragging with the middle mouse button, as well as by using the sliders.
- Coordinate grid can be turned on/off.
- The sphere can be zoomed in/out using the mouse wheel or the slider, to a minimum of 50% and a maximum of 200%.
- On having a point selected, the coordinates of the point are displayed. The point can be configured to be non-movable or non-removable.
- If a single point having an associated great circle is selected, properties of the great circle are displayed.
- If 3 points are selected, the properties of the triangle formed by them are displayed.
- Diagrams can be saved as .json files, which can be loaded later. A fresh new diagram can be obtained. These options are available at the bottom left in the file panel.
