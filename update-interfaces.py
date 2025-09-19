import shutil
import subprocess
import xml.etree.ElementTree as ET
from pathlib import Path

# --- Repository Definitions ---
FLATPAK_REPO_URL = "https://github.com/flatpak/flatpak.git"
FLATPAK_TEMP_DIR = Path("/tmp/flatpak-interfaces")
XDG_PORTAL_REPO_URL = "https://github.com/flatpak/xdg-desktop-portal.git"
XDG_PORTAL_TEMP_DIR = Path("/tmp/xdg-portal-interfaces")

# --- Output Definitions ---
PORTAL_OUTPUT_DIR = Path("interfaces")
IMPL_OUTPUT_DIR = Path("interfaces/backend")

# Standard XML header
XML_HEADER = """<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
                    "http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">"""


def format_xml_content(unformatted_xml):
    """
    Uses xmllint to pretty-print XML content passed as a string.
    Returns the formatted string.
    """
    try:
        result = subprocess.run(
            ["xmllint", "--format", "-"],
            input=unformatted_xml.encode("utf-8"),
            capture_output=True,
            check=True,
        )
        return result.stdout.decode("utf-8")
    except (FileNotFoundError, subprocess.CalledProcessError) as e:
        print(f"Error calling xmllint: {e}. Returning unformatted XML.")
        return unformatted_xml


def remove_all_annotations(element):
    """
    Recursively removes all <annotation> tags from the XML tree.
    """
    children_to_remove = []
    for child in list(element):
        if child.tag == "annotation":
            children_to_remove.append(child)
        else:
            remove_all_annotations(child)

    for child in children_to_remove:
        element.remove(child)


def process_and_format_each_file(pattern, source_dir, output_dir):
    """
    Finds XML files, formats each one individually, and writes to a new file.
    """
    print(
        f"Processing pattern for INDIVIDUAL files: {pattern} from {source_dir}"
    )
    output_dir.mkdir(parents=True, exist_ok=True)
    found_files = list(source_dir.glob(f"data/{pattern}"))

    if not found_files:
        print(
            f"No files found for pattern '{pattern}'. Skipping individual processing."
        )
        return

    for file_path in found_files:
        output_file = output_dir / file_path.name
        print(f"  - Processing: {file_path} -> {output_file}")

        try:
            tree = ET.parse(file_path)
            root = tree.getroot()

            # Remove all annotation tags from the entire tree
            remove_all_annotations(root)

            # Create a new, empty node to hold processed interfaces
            processed_node = ET.Element("node")

            for interface in root.findall("interface"):
                # Exclude the org.freedesktop.DBus.Peer interface
                if interface.get("name") == "org.freedesktop.DBus.Peer":
                    continue

                # Append the cleaned interface to the new node
                processed_node.append(interface)

            unformatted_xml = ET.tostring(
                processed_node, encoding="unicode", xml_declaration=False
            )
            formatted_xml = format_xml_content(unformatted_xml)
            final_output = f"{XML_HEADER}\n{formatted_xml}"

            with open(output_file, "w") as f:
                f.write(final_output)
        except ET.ParseError as e:
            print(f"Error parsing XML file {file_path}: {e}")
            continue

    print(f"Finished processing individual files for pattern '{pattern}'")


def main():
    """Main function to run the update process."""
    # Clean previous output directories to avoid stale files
    if PORTAL_OUTPUT_DIR.exists():
        shutil.rmtree(PORTAL_OUTPUT_DIR)
    if IMPL_OUTPUT_DIR.exists():
        shutil.rmtree(IMPL_OUTPUT_DIR)

    # Clean temporary directories
    if FLATPAK_TEMP_DIR.exists():
        shutil.rmtree(FLATPAK_TEMP_DIR)
    if XDG_PORTAL_TEMP_DIR.exists():
        shutil.rmtree(XDG_PORTAL_TEMP_DIR)

    # --- Part 1: Clone both repositories ---
    print(f"Cloning {XDG_PORTAL_REPO_URL} to {XDG_PORTAL_TEMP_DIR}...")
    subprocess.run(
        [
            "git",
            "clone",
            "--depth",
            "1",
            XDG_PORTAL_REPO_URL,
            str(XDG_PORTAL_TEMP_DIR),
        ],
        check=True,
    )

    print(f"Cloning {FLATPAK_REPO_URL} to {FLATPAK_TEMP_DIR}...")
    subprocess.run(
        [
            "git",
            "clone",
            "--depth",
            "1",
            FLATPAK_REPO_URL,
            str(FLATPAK_TEMP_DIR),
        ],
        check=True,
    )

    # --- Part 2: Process the files individually from both repositories ---

    # Process portal files from xdg-desktop-portal repo
    process_and_format_each_file(
        "org.freedesktop.portal.*.xml", XDG_PORTAL_TEMP_DIR, PORTAL_OUTPUT_DIR
    )

    # Process impl files from xdg-desktop-portal repo
    process_and_format_each_file(
        "org.freedesktop.impl.portal.*.xml",
        XDG_PORTAL_TEMP_DIR,
        IMPL_OUTPUT_DIR,
    )

    # Process portal files from flatpak repo
    process_and_format_each_file(
        "org.freedesktop.portal.*.xml", FLATPAK_TEMP_DIR, PORTAL_OUTPUT_DIR
    )

    # Process impl files from flatpak repo
    process_and_format_each_file(
        "org.freedesktop.impl.portal.*.xml", FLATPAK_TEMP_DIR, IMPL_OUTPUT_DIR
    )

    print("Cleanup complete.")


if __name__ == "__main__":
    main()
