using UnityEngine;
using Mapbox.Unity.Map;

#if ENABLE_INPUT_SYSTEM
using UnityEngine.InputSystem;
using UnityEngine.InputSystem.UI;
using UnityEngine.InputSystem.Controls;
#else
using UnityEngine.EventSystems;
#endif

namespace Joby.Autonomy.FleetVisualization
{
    public class ThirdPersonCamera : MonoBehaviour {
        public const int MOUSE_PAN_BUTTON = 0;
        public const int MOUSE_ORBIT_BUTTON = 1;

        public float pan_smoothing = 8f;
        public float orbit_smoothing = 8f;
        public float zoom_smoothing = 5f;

        // public float FollowStrength = 1f;   // determines responsiveness of target following

        // [Range(1f, 90f)]
        // public float MaxFollowDeltaAngle = 60f;  // prevents the camera from trying to move too fast

        // [Range(1f, 90f)]
        // public float FollowAngleDeadZone = 20f;  // angle at which following is ~ 0.5x maximum speed

        public float pan_sensitivity = 8f;
        public float zoom_sensitivity = 0.2f;
        public float orbit_sensitivity_x = 0.4f;
        public float orbit_sensitivity_y = 0.3f;

        public float max_distance = 10000f;
        public float min_distance = 50f;
        public float min_zoom = 1f;
        public float max_zoom = 20f;
        public float min_theta = -80f;
        public float max_theta = 80f;

        public float MinX = -200f;
        public float MaxX = 200f;
        public float MinZ = -200f;
        public float MaxZ = 200f;

        public float FieldOfView = 40f;

        /// <summary>
        /// If we're right clicking to rotate the camera, that mouseup shouldn't
        /// cancel any tools or whatever else right click is supposed to do.
        /// however, if we happen to move the mouse just a little, it still counts.
        /// </summary>
        public float right_click_threshhold = 20f;

        public static bool IgnoreRightClick = false;

        public string FocusOnCentroidWithTag = "";
        public Transform FocusObject;
        public AbstractMap MapboxMap;
        public MapAdapter MapAdapter;
        public bool LockPositionToObject = true;
        public bool FollowObjectYaw = true;
        public bool LookTowardsObject = true;

        public InputSystemUIInputModule InputSystemUI;

        private Vector3 _focusPositionTarget;
        private Vector3 _panOffsetWorldSpace;
        private Vector3 _panOffsetTarget;
        private Vector3 _dragStartPoint;
        private Mapbox.Utils.Vector2d _dragStartLatLon;
        private Vector2 _rightClickStart;
        private Camera _cam;
        private float _zoomLevelTarget;
        private float _currentZoomLevel;
        private Vector3 _currentEulerAngles;
        private Vector3 _eulerAnglesTargetDelta;

        private Plane _groundPlane;

        // Prevents panning/rotating when interacting with the UI.
        private bool _mouseEnabled = false;
        private bool _panningStarted = false;
        private bool _orbitStarted = false;

        #if !ENABLE_INPUT_SYSTEM
        private bool _mousePanButton = false;
        private bool _mousePanButtonDown = false;
        private bool _mouseOrbitButton = false;
        private bool _mouseOrbitButtonDown = false;
        #endif


        // Used for mouse tracking.
        private Vector3 _lastMousePosition = Vector3.zero;
        private Vector2 _mouseDelta = Vector2.zero;

        // TODO: there appears to be a unity bug when clicking BTN 1, then BTN 2,
        // then releasing BTN 2, BTN 1 outside the window. The state of BTN 1 as
        // reported by Input.GetMouseButton() never changes. idiots!

        private float DistanceToZoomLevel(float distance)
        {
            return -Mathf.Log(distance);
        }

        private float ZoomLevelToDistance(float zoomLevel)
        {
            return Mathf.Exp(-zoomLevel);
        }

        public void Start() {
            _cam = GetComponentInChildren<Camera>();
            _rightClickStart = Vector2.zero;
            _currentZoomLevel = DistanceToZoomLevel(-_cam.transform.localPosition.z);
            _zoomLevelTarget = _currentZoomLevel;
            _currentEulerAngles = transform.rotation.eulerAngles;
            _eulerAnglesTargetDelta = Vector3.zero;
            _cam.fieldOfView = FieldOfView;

            _groundPlane = new Plane(Vector3.up, Vector3.zero);

            if (MapboxMap != null)
            {
                _dragStartLatLon = MapboxMap.CenterLatitudeLongitude;
                _currentZoomLevel = MapboxMap.Zoom;
                _zoomLevelTarget = _currentZoomLevel;
            }
        }

        public void Update()
        {
            // Calculate mouse deltas manually since the idiotic built-in API scales it by the user
            // sensitivity settings, which is neither deterministic nor accessible programmatically.

            #if !ENABLE_INPUT_SYSTEM
            _mouseDelta = Input.mousePosition - _lastMousePosition;
            _lastMousePosition = Input.mousePosition;

            bool ctrlKey = (
                Input.GetKey(KeyCode.LeftControl) ||
                Input.GetKey(KeyCode.RightControl) ||
                Input.GetKey(KeyCode.LeftCommand) ||
                Input.GetKey(KeyCode.RightCommand)
            );

            bool ctrlKeyDown = (
                Input.GetKeyDown(KeyCode.LeftControl) ||
                Input.GetKeyDown(KeyCode.RightControl) ||
                Input.GetKeyDown(KeyCode.LeftCommand) ||
                Input.GetKeyDown(KeyCode.RightCommand)
            );

            bool ctrlKeyUp = (
                Input.GetKeyUp(KeyCode.LeftControl) ||
                Input.GetKeyUp(KeyCode.RightControl) ||
                Input.GetKeyUp(KeyCode.LeftCommand) ||
                Input.GetKeyUp(KeyCode.RightCommand)
            );

            _mousePanButton = Input.GetMouseButton(MOUSE_PAN_BUTTON) && !ctrlKey;
            _mousePanButtonDown = (Input.GetMouseButtonDown(MOUSE_PAN_BUTTON) && !ctrlKey) || (Input.GetMouseButton(MOUSE_PAN_BUTTON) && ctrlKeyUp);
            _mouseOrbitButton = Input.GetMouseButton(MOUSE_ORBIT_BUTTON) || (Input.GetMouseButton(MOUSE_PAN_BUTTON) && ctrlKey);
            _mouseOrbitButtonDown = Input.GetMouseButtonDown(MOUSE_ORBIT_BUTTON) || (Input.GetMouseButtonDown(MOUSE_PAN_BUTTON) && ctrlKey) || Input.GetMouseButton(MOUSE_PAN_BUTTON) && ctrlKeyDown;
            #endif
        }

        public void LateUpdate()
        {
            #if ENABLE_INPUT_SYSTEM
            _mouseEnabled = InputSystemUI == null ? true : !InputSystemUI.IsPointerOverGameObject(-1);
            #else
            _mouseEnabled = !EventSystem.current.IsPointerOverGameObject(-1);
            #endif

            if (!LockPositionToObject)
            {
                UpdatePositionTarget();
            }
            UpdateZoomTarget();
            UpdateRotationTarget();
            if (LookTowardsObject)
            {
                LookTowardsTarget();
            }
            CheckRightClickThreshhold();

            RotateTowardsTarget();
            // ZoomTowardsTarget();
            bool positionChanged = UpdatePosition();
            bool zoomChanged =  UpdateZoom();
            if (MapboxMap != null && (positionChanged || zoomChanged))
            {
                MapboxMap.UpdateMap();
                MapAdapter.UpdateMapScale();
            }
        }

        private void RotateTowardsTarget()
        {
            // update camera rotation
            Vector3 eulerAnglesDelta = _eulerAnglesTargetDelta * (orbit_smoothing * Mathf.Min(Time.deltaTime, 0.02f));
            _eulerAnglesTargetDelta -= eulerAnglesDelta;
            // Vector3 currentEulerAngles = transform.rotation.eulerAngles;

            if (_currentEulerAngles.x > 180f)
            {
                // We define the lower limit as a negative angle, but the eulerAngles property
                // returns a 0-360 range so we need to wrap it around in order to clamp the value.
                _currentEulerAngles.x -= 360f;
            }
            else if (_currentEulerAngles.x > 90f)
            {
                // It shouldn't be possible to get into this upside-down angle range,
                // but if somehow it happens just reset to 0...
                _currentEulerAngles.x = 0f;
            }

            _currentEulerAngles += eulerAnglesDelta;

            // Constrain the vertical angle.
            _currentEulerAngles.x = Mathf.Clamp(_currentEulerAngles.x, min_theta, max_theta);

            // Prevent horizontal angle from going too far off the deep end.
            _currentEulerAngles.y = _currentEulerAngles.y % 360f;


            // Now add the target's yaw since we want to automatically follow as it turns.
            Vector3 absoluteEulerAngles = _currentEulerAngles;
            if (FocusObject != null && FollowObjectYaw)
            {
                absoluteEulerAngles.y += FocusObject.rotation.eulerAngles.y;
            }

            transform.rotation = Quaternion.Euler(absoluteEulerAngles);
        }

        private Mapbox.Utils.Vector2d GetLatLonOffsetFromUnityOffset(Vector3 offsetWorldSpace)
        {
            // See AbstractMap.WorldToGeoPosition()
            // For quadtree implementation of the map, the map scale needs to be compensated for.
            var scaleFactor = Mathf.Pow(2f, (MapboxMap.InitialZoom - MapboxMap.AbsoluteZoom));
            Vector3 offsetLocalMapboxObject = MapboxMap.Root.InverseTransformPoint(offsetWorldSpace) / (MapboxMap.WorldRelativeScale * scaleFactor);
			return Mapbox.Unity.Utilities.Conversions.MetersToLatLon(
                new Mapbox.Utils.Vector2d(offsetLocalMapboxObject.x, offsetLocalMapboxObject.z)
            );
        }

        /// <summary>
        /// Update position of camera focus point.
        /// </summary>
        /// <returns>True if the position changed.</returns>
        private bool UpdatePosition()
        {
            // TODO: refactor this to use MapAdapter.SetMapCenterGeodetic()
            if (FocusObject != null && LockPositionToObject)
            {
                bool positionError = transform.position != FocusObject.position;
                transform.position = FocusObject.position;
                return positionError;
            }
            else if (MapboxMap != null)
            {
                // If we get close enough, stop updating.
                Vector3 offsetError = _panOffsetWorldSpace - _panOffsetTarget;
                if (Mathf.Abs(offsetError.x) + Mathf.Abs(offsetError.y) + Mathf.Abs(offsetError.z) < 0.0001f) { return false; }

                _panOffsetWorldSpace = Vector3.Lerp(_panOffsetWorldSpace, _panOffsetTarget, pan_smoothing * Mathf.Min(Time.deltaTime, 0.02f));
                Mapbox.Utils.Vector2d newLatLon = _dragStartLatLon + GetLatLonOffsetFromUnityOffset(_panOffsetWorldSpace);
                MapboxMap.SetCenterLatitudeLongitude(newLatLon);
            }
            else
            {
                Vector3 positionError = transform.position - _focusPositionTarget;
                if (Mathf.Abs(positionError.x) + Mathf.Abs(positionError.y) + Mathf.Abs(positionError.z) < 0.00001f) { return false; }

                transform.position = Vector3.Lerp(transform.position, _focusPositionTarget, pan_smoothing * Mathf.Min(Time.deltaTime, 0.02f));
            }
            return true;
        }

        private void UpdatePositionTarget()
        {
            #if ENABLE_INPUT_SYSTEM
            ButtonControl panButton;
            if (!Mouse.current.rightButton.isPressed && Keyboard.current.ctrlKey.isPressed)
            {
                panButton = Mouse.current.leftButton;
            }
            else
            {
                panButton = Mouse.current.rightButton;
            }
            if (panButton.isPressed)
            #else
            // if (Input.GetMouseButton(1))
            if (_mousePanButton)
            #endif
            {
                #if ENABLE_INPUT_SYSTEM
                if (panButton.wasPressedThisFrame && _mouseEnabled)
                #else
                // if (Input.GetMouseButtonDown(1) && _mouseEnabled)
                if (_mousePanButtonDown && _mouseEnabled)
                #endif
                {
                    _panningStarted = true;
                }

                if (_panningStarted)
                {
                    #if ENABLE_INPUT_SYSTEM
                    Vector2 mousePosition = Mouse.current.position.ReadValue();
                    #else
                    Vector2 mousePosition = Input.mousePosition;
                    #endif

                    Ray clickRay = Camera.main.ScreenPointToRay(mousePosition);
                    float dist;
                    if (_groundPlane.Raycast(clickRay, out dist))
                    {
                        Vector3 gridPoint = clickRay.GetPoint(dist);

                        #if ENABLE_INPUT_SYSTEM
                        if (panButton.wasPressedThisFrame)
                        #else
                        // if (Input.GetMouseButtonDown(1))
                        if (_mousePanButtonDown)
                        #endif
                        {
                            _dragStartPoint = gridPoint;
                            // change cursor?
                            if (MapboxMap != null)
                            {
                                _dragStartLatLon = MapboxMap.CenterLatitudeLongitude;
                                _panOffsetWorldSpace = Vector3.zero;
                            }
                        }
                        if (MapboxMap != null)
                        {
                            _panOffsetTarget = _dragStartPoint - gridPoint;
                        }
                        else
                        {
                            _focusPositionTarget = transform.position + _dragStartPoint - gridPoint;
                            _focusPositionTarget.x = Mathf.Clamp(_focusPositionTarget.x, this.MinX, this.MaxX);
                            _focusPositionTarget.z = Mathf.Clamp(_focusPositionTarget.z, this.MinZ, this.MaxZ);
                        }
                    }
                }
            }
            else
            {
                _panningStarted = false;
            }
        }

        /// <summary>
        /// Update camera zoom distance.
        /// </summary>
        /// <returns>True if the zoom changed.</returns>
        private bool UpdateZoom()
        {
            // TODO: refactor this to use MapAdapter.SetMapCenterGeodetic()

            // If we get close enough, stop updating.
            if (Mathf.Abs(_currentZoomLevel - _zoomLevelTarget) < 0.0001f) { return false; }
            _currentZoomLevel = Mathf.Lerp(_currentZoomLevel, _zoomLevelTarget, zoom_smoothing * Mathf.Min(Time.deltaTime, 0.02f));

            if (MapboxMap == null)
            {
                Vector3 newPosition = _cam.transform.localPosition;
                newPosition.z = -ZoomLevelToDistance(_currentZoomLevel);
                _cam.transform.localPosition = newPosition;
            }
            else
            {
                MapboxMap.SetZoom(_currentZoomLevel);
            }

            return true;
        }

        private void UpdateZoomTarget()
        {
            if (_mouseEnabled)
            {
                #if ENABLE_INPUT_SYSTEM
                #if UNITY_STANDALONE_LINUX
                float mouseWheel = -Mouse.current.scroll.y.ReadValue();
                #else
                float mouseWheel = Mouse.current.scroll.y.ReadValue();
                #endif
                if (mouseWheel < 0) { _zoomLevelTarget -= zoom_sensitivity; }
                if (mouseWheel > 0) { _zoomLevelTarget += zoom_sensitivity; }
                #else
                float mouseWheel = -Input.mouseScrollDelta.y;
                // zoom in
                if (Input.GetKeyDown(KeyCode.Equals) || Input.GetKeyDown(KeyCode.Plus) || mouseWheel < 0) { _zoomLevelTarget += zoom_sensitivity; }
                // zoom out
                else if (Input.GetKeyDown(KeyCode.Minus) || mouseWheel > 0) { _zoomLevelTarget -= zoom_sensitivity; }
                #endif

                if (MapboxMap != null)
                {
                    _zoomLevelTarget = Mathf.Clamp(_zoomLevelTarget, min_zoom, max_zoom);
                }
                else
                {
                    _zoomLevelTarget = Mathf.Clamp(_zoomLevelTarget, DistanceToZoomLevel(max_distance), DistanceToZoomLevel(min_distance));
                }
            }
        }

        private void UpdateRotationTarget()
        {
            #if ENABLE_INPUT_SYSTEM
            if (Mouse.current.leftButton.isPressed && !Keyboard.current.ctrlKey.isPressed)
            #else
            // if (Input.GetMouseButton(0))
            if (_mouseOrbitButton)
            #endif
            {
                #if ENABLE_INPUT_SYSTEM
                if (Mouse.current.leftButton.wasPressedThisFrame && _mouseEnabled)
                #else
                // if (Input.GetMouseButtonDown(0) && _mouseEnabled)
                if (_mouseOrbitButtonDown && _mouseEnabled)
                #endif
                {
                    _orbitStarted = true;
                }

                if (_orbitStarted)
                {
                    #if ENABLE_INPUT_SYSTEM
                    _eulerAnglesTargetDelta.x -= Mouse.current.delta.y.ReadValue() * orbit_sensitivity_y;
                    _eulerAnglesTargetDelta.y += Mouse.current.delta.x.ReadValue() * orbit_sensitivity_x;
                    #else
                    _eulerAnglesTargetDelta.x -= _mouseDelta.y * orbit_sensitivity_y;
                    _eulerAnglesTargetDelta.y += _mouseDelta.x * orbit_sensitivity_x;
                    #endif
                }
            }
            else
            {
                _orbitStarted = false;
            }
        }

        /// <summary>
        /// Swivel the camera to face the focus target.
        /// This is a completely separate process from user-directed rotation and panning.
        /// </summary>
        private void LookTowardsTarget()
        {
            Vector3 targetPosition;
            if (FocusObject != null && !LockPositionToObject)
            {
                targetPosition = FocusObject.position;
            }
            else if (FocusOnCentroidWithTag != "")
            {
                GameObject[] objects = GameObject.FindGameObjectsWithTag(FocusOnCentroidWithTag);
                if (objects.Length > 0)
                {
                    targetPosition = Vector3.zero;
                    foreach (GameObject obj in objects)
                    {
                        targetPosition += obj.transform.position;
                    }
                    targetPosition /= (float)objects.Length;
                    }
                    else
                {
                    return;
                }
            }
            else
            {
                return;
            }

            // For this motion we want the camera to stay fixed, but the camera focus needs to
            // swing towards the target, rotating about the camera so the camera doesn't appear
            // to translate. This is the opposite of normal orbit mode.
            Vector3 cameraToTarget = targetPosition - _cam.transform.position;
            Vector2 cameraToTargetHorizontal = new Vector2(cameraToTarget.x, cameraToTarget.z);

            Vector3 cameraLookDirection = _cam.transform.forward;
            Vector2 cameraLookDirectionHorizontal = new Vector2(cameraLookDirection.x, cameraLookDirection.z);

            // Compute angle from current camera direction to the focus object
            float yawDeltaToTarget = -Vector2.SignedAngle(cameraLookDirectionHorizontal, cameraToTargetHorizontal);
            // Apply an easing function so the camera doesn't obsessively keep the target in the
            // exact center of the screen
            yawDeltaToTarget = MaxFollowDeltaAngle * (float)System.Math.Tanh(Mathf.Pow(yawDeltaToTarget / FollowAngleDeadZone, 3f));
            yawDeltaToTarget *= FollowStrength * Mathf.Min(Time.deltaTime, 0.02f);

            // Compute a position delta to rotate the focus point about the camera's position
            Vector3 cameraArmHorizontal = Vector3.ProjectOnPlane(transform.position - _cam.transform.position, Vector3.up);
            Vector3 focusPointDisplacement = (Quaternion.AngleAxis(yawDeltaToTarget, Vector3.up) * cameraArmHorizontal) - cameraArmHorizontal;

            if (_panningStarted)
            {
                _dragStartPoint += focusPointDisplacement;
            }
            transform.position += focusPointDisplacement;
            _focusPositionTarget += focusPointDisplacement;
            // This feels awkward, but it would be inefficient to compute euler angles and
            // quaternions both here and in RotateTowardsTarget()
            _currentEulerAngles.y += yawDeltaToTarget;
        }

        /// <summary>
        /// track how far the mouse has been dragged,
        /// so we know when to stop recognizing mouseUp as a click
        /// </summary>
        private void CheckRightClickThreshhold()
        {
            #if ENABLE_INPUT_SYSTEM
            if (Mouse.current.rightButton.isPressed)
            #else
            // if (Input.GetMouseButton(1))
            if (_mousePanButton)
            #endif
            {
                #if ENABLE_INPUT_SYSTEM
                Vector2 mousePosition = Mouse.current.position.ReadValue();
                if (Mouse.current.rightButton.wasPressedThisFrame)
                #else
                Vector2 mousePosition = Input.mousePosition;
                // if (Input.GetMouseButtonDown(1))
                if (_mousePanButtonDown)
                #endif
                {
                    _rightClickStart = mousePosition;
                }

                float mouseDist = Mathf.Abs(mousePosition.x - _rightClickStart.x) +
                    Mathf.Abs(mousePosition.y - _rightClickStart.y);

                if (!ThirdPersonCamera.IgnoreRightClick && mouseDist > right_click_threshhold)
                {
                    ThirdPersonCamera.IgnoreRightClick = true;
                }

            }
            #if ENABLE_INPUT_SYSTEM
            else if (!Mouse.current.rightButton.wasReleasedThisFrame)
            #else
            // else if (!Input.GetMouseButtonDown(1))
            else if (!_mousePanButtonDown)
            #endif
            {
                // we want to leave this true for the frame the mouse is released, otherwise it would be pointless
                ThirdPersonCamera.IgnoreRightClick = false;
            }
            // if (Input.GetMouseButton(1))
            // {
            //     if (Input.GetMouseButtonDown(1))
            //     {
            //         rightClickStart = Input.mousePosition;
            //     }

            //     float mouseDist = Mathf.Abs(Input.mousePosition.x - rightClickStart.x) +
            //         Mathf.Abs(Input.mousePosition.y - rightClickStart.y);

            //     if (!ThirdPersonCamera.ignoreRightClick && mouseDist > rightClickThreshhold)
            //     {
            //         ThirdPersonCamera.ignoreRightClick = true;
            //     }

            // }
            // else if (!Input.GetMouseButtonUp(1))
            // {
            //     // we want to leave this true for the frame the mouse is released, otherwise it would be pointless
            //     ThirdPersonCamera.ignoreRightClick = false;
            // }
        }

        public void OnDrawGizmos()
        {

        }
    }
}
