# Celestial Sphere

[Celestial Sphere](https://bunchofcellulose.github.io/celestialsphere/) is a web application that provides a neat and user-friendly interface for drawing and visualizing diagrams on a sphere. My aim for building this is to provide a tool for students and educators to easily learn and engage with spherical diagrams, which are commonly used in spherical astronomy.

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
- `Shift` + `.` to draw a great circle with the 2 selected points lying on it.
- `,` to draw a small circle with the 3 selected points lying on it.
- `Shift` + `,` to draw a small circle with the first selected point as the pole and second selected point as a point on the small circle.
- `Shift` + type to name the great circle/small circle, while the associated pole point is selected.
- `Shift` + move to snap a point onto a nearby great circle.
- `Shift` + Left click on the sphere to add a point on a nearby great circle.
- `Ctrl` + `h` to hide/show the selected point(s).
- `Ctrl` + `g` to group the selected points.
- `Ctrl` + `u` to ungroup the selected points.
- `Ctrl` + `a` to rotate the sphere in the x direction.
- `Ctrl` + `s` to rotate the sphere in the y direction.
- `Ctrl` + `d` to rotate the sphere in the z direction.

## Features

- The rotation of the sphere can be controlled by dragging with the middle mouse button, as well as by using the sliders.
- Coordinate grid can be turned on/off.
- The sphere can be zoomed in/out using the mouse wheel or the slider, to a minimum of 50% and a maximum of 200%.
- On having a point selected, the coordinates of the point are displayed. The point can be configured to be non-movable or non-removable.
- If a single point having an associated great circle is selected, properties of the great circle are displayed.
- If 3 points are selected, the properties of the triangle formed by them are displayed.
- Diagrams can be saved as .json files, which can be loaded later. The diagrams can also be saved as .svg files. A fresh new diagram can be obtained. These options are available at the bottom left in the file panel.
- Small circle having the same pole as a great circle can not be renamed, the great circle has to be first removed, then the small circle can be renamed.
- Points can be hidden, which will not be displayed on the sphere. This can be toggled by using the checkbox on the top left panel.
- Points can be grouped, which will allow for easier manipulation of multiple points at once. The grouped points move together, and can be renamed as a group. Ungrouping will remove the group but keep the points intact.

## TODO

- Add a method to place a point at a distance from a selected point
- Add a method to place a point at an angle to selected points
- Add a method to place a point at a distance and angle from selected points
- Add a method to place point at a given coordinate.
- Add features to snap to circle intersections, small circles and arcs.
- Add feature to add multiple small circles with one pole.
