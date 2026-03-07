#include "dcel.h"

void dcel::DCEL::getOuterComponents(Face f, std::vector<HalfEdge> &edgeList) {
	HalfEdge h = outerComponent(f);
	Ref startid = h.id;

	do {
		edgeList.push_back(h);
		h = next(h);
	} while (h.id != startid);
}

void dcel::DCEL::getOuterComponents(Face f, std::vector<Ref> &edgeList) {
	HalfEdge h = outerComponent(f);
	Ref startid = h.id;

	do {
		edgeList.push_back(h.id);
		h = next(h);
	} while (h.id != startid);
}

void dcel::DCEL::getIncidentEdges(Vertex v, std::vector<HalfEdge> &edgeList) {
	HalfEdge h = incidentEdge(v);
	Ref startid = h.id;

	do {
		edgeList.push_back(h);
		h = next(twin(h));
	} while (h.id != startid);
}

void dcel::DCEL::getIncidentEdges(Vertex v, std::vector<Ref> &edgeList) {
	HalfEdge h = incidentEdge(v);
	Ref startid = h.id;

	do {
		edgeList.push_back(h.id);
		h = next(twin(h));
	} while (h.id != startid);
}

void dcel::DCEL::getIncidentFaces(Vertex v, std::vector<Face> &faceList) {
	HalfEdge h = incidentEdge(v);
	Ref startid = h.id;

	do {
		if (!isBoundary(h)) {
			faceList.push_back(incidentFace(h));
		}
		h = next(twin(h));
	} while (h.id != startid);
}

void dcel::DCEL::getIncidentFaces(Vertex v, std::vector<Ref> &faceList) {
	HalfEdge h = incidentEdge(v);
	Ref startid = h.id;

	do {
		if (!isBoundary(h)) {
			faceList.push_back(incidentFace(h).id);
		}
		h = next(twin(h));
	} while (h.id != startid);
}

bool dcel::DCEL::isBoundary(HalfEdge h) {
	return h.incidentFace.ref == -1;
}
