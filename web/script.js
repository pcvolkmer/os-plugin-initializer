import * as styles from './style.css';

function handleDownload() {
    let project_type = document.getElementById('maven').checked ? 'maven' : 'gradle';
    let os_version = document.getElementById('2.14.0').checked ? '2.14.0' : '2.13.2';
    let group = document.getElementById('group').value
        .toLowerCase()
        .replaceAll(/\.+/g, '.')
        .replaceAll(/[^a-z0-9.]|^\.+|\.+$/g, '');
    let artifact = document.getElementById('artifact').value
        .toLowerCase()
        .replaceAll(/\.+/g, '.')
        .replaceAll(/[^a-z0-9.]|^\.+|\.+$/g, '');
    let description = document.getElementById('description').value;
    let package_name = document.getElementById('package_name').value
        .toLowerCase()
        .replaceAll(/\.+/g, '.')
        .replaceAll(/[^a-z0-9.]|^\.+|\.+$/g, '');
    const url = `project.zip?project_type=${project_type}&os_version=${os_version}&group=${group}&artifact=${artifact}&description=${description}&package_name=${package_name}`;
    fetch(url).then(res => res.blob()).then(blob => {
        const a = document.getElementById('blob');
        const url = URL.createObjectURL(blob);
        a.href = url;
        a.download = `${artifact}.zip`;
        a.click();
        URL.revokeObjectURL(url);
    });
}
document.addEventListener('keydown', (e) => {
    if (e.ctrlKey && e.key === 'Enter') {
        document.getElementById('submit').focus();
        handleDownload();
    }
});
document.getElementById('submit').addEventListener('click', (e) => {
    handleDownload();
});
document.getElementById('group').addEventListener('input', (e) => {
    let group = e.target.value;
    let artifact = document.getElementById('artifact').value;
    document.getElementById('package_name').value = `${group}.${artifact}`
        .toLowerCase()
        .replaceAll(/\.+/g, '.')
        .replaceAll(/[^a-z0-9.]|^\.+|\.+$/g, '');
});

document.getElementById('artifact').addEventListener('input', (e) => {
    let artifact = e.target.value;
    let group = document.getElementById('group').value;
    document.getElementById('package_name').value = `${group}.${artifact}`
        .toLowerCase()
        .replaceAll(/\.+/g, '.')
        .replaceAll(/[^a-z0-9.]|^\.+|\.+$/g, '');
});